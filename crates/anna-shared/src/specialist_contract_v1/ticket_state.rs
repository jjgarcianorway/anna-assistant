//! Ticket States and Stats (Part E) - v0.0.440.
//!
//! Ticket lifecycle states for accurate stats tracking.
//!
//! States:
//! - OPEN: Ticket created, processing
//! - RESOLVED: Required evidence succeeded, valid answer, confidence >= threshold
//! - FAILED_PROBE: Required probes failed
//! - FAILED_SPECIALIST: Specialist invalid after retries AND fallback failed
//! - NEED_CLARIFICATION: User input needed
//! - ESCALATED: Needs human intervention
//!
//! Stats integrity:
//! - resolved = count of RESOLVED only
//! - success_rate = resolved / total_tickets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ticket lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketState {
    /// Ticket created, processing.
    Open,
    /// Successfully resolved with valid answer.
    Resolved,
    /// Required probes failed.
    FailedProbe,
    /// Specialist invalid after retries AND fallback failed.
    FailedSpecialist,
    /// User input needed to proceed.
    NeedClarification,
    /// Needs human intervention.
    Escalated,
}

impl TicketState {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Resolved => "RESOLVED",
            Self::FailedProbe => "FAILED_PROBE",
            Self::FailedSpecialist => "FAILED_SPECIALIST",
            Self::NeedClarification => "NEED_CLARIFICATION",
            Self::Escalated => "ESCALATED",
        }
    }

    /// Check if terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Resolved | Self::FailedProbe | Self::FailedSpecialist | Self::Escalated
        )
    }

    /// Check if success state.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Resolved)
    }

    /// Check if failure state.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::FailedProbe | Self::FailedSpecialist)
    }
}

impl Default for TicketState {
    fn default() -> Self {
        Self::Open
    }
}

/// Minimum confidence threshold for RESOLVED.
pub const MIN_CONFIDENCE_FOR_RESOLVED: f64 = 0.5;

/// Resolution criteria for transitioning to RESOLVED.
#[derive(Debug, Clone)]
pub struct ResolutionCriteria {
    /// All required evidence collected.
    pub evidence_complete: bool,
    /// Valid answer produced (from specialist or fallback).
    pub valid_answer: bool,
    /// Confidence level.
    pub confidence: f64,
    /// Confidence threshold (default 0.5).
    pub threshold: f64,
}

impl ResolutionCriteria {
    /// Create new criteria.
    pub fn new(evidence_complete: bool, valid_answer: bool, confidence: f64) -> Self {
        Self {
            evidence_complete,
            valid_answer,
            confidence,
            threshold: MIN_CONFIDENCE_FOR_RESOLVED,
        }
    }

    /// Set custom threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Check if criteria met for RESOLVED.
    pub fn is_resolved(&self) -> bool {
        self.evidence_complete && self.valid_answer && self.confidence >= self.threshold
    }

    /// Get reason if not resolved.
    pub fn failure_reason(&self) -> Option<&'static str> {
        if !self.evidence_complete {
            Some("Required evidence not collected")
        } else if !self.valid_answer {
            Some("No valid answer produced")
        } else if self.confidence < self.threshold {
            Some("Confidence below threshold")
        } else {
            None
        }
    }
}

/// State transition event.
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Previous state.
    pub from: TicketState,
    /// New state.
    pub to: TicketState,
    /// Reason for transition.
    pub reason: String,
    /// Timestamp (milliseconds since epoch).
    pub timestamp_ms: u64,
}

impl StateTransition {
    /// Create a new transition.
    pub fn new(from: TicketState, to: TicketState, reason: &str) -> Self {
        Self {
            from,
            to,
            reason: reason.to_string(),
            timestamp_ms: current_timestamp_ms(),
        }
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// State machine for ticket lifecycle.
#[derive(Debug, Clone)]
pub struct TicketStateMachine {
    /// Current state.
    pub state: TicketState,
    /// Transition history.
    pub history: Vec<StateTransition>,
    /// Case ID.
    pub case_id: String,
}

impl TicketStateMachine {
    /// Create new state machine for a ticket.
    pub fn new(case_id: &str) -> Self {
        Self {
            state: TicketState::Open,
            history: Vec::new(),
            case_id: case_id.to_string(),
        }
    }

    /// Transition to RESOLVED.
    pub fn resolve(&mut self, criteria: &ResolutionCriteria) -> Result<(), &'static str> {
        if !criteria.is_resolved() {
            return Err(criteria.failure_reason().unwrap_or("Unknown reason"));
        }

        let transition = StateTransition::new(
            self.state,
            TicketState::Resolved,
            &format!("Resolved with confidence {:.2}", criteria.confidence),
        );
        self.history.push(transition);
        self.state = TicketState::Resolved;
        Ok(())
    }

    /// Transition to FAILED_PROBE.
    pub fn fail_probe(&mut self, failed_probes: &[&str]) {
        let reason = if failed_probes.is_empty() {
            "Required probes failed".to_string()
        } else {
            format!("Probes failed: {}", failed_probes.join(", "))
        };

        let transition = StateTransition::new(self.state, TicketState::FailedProbe, &reason);
        self.history.push(transition);
        self.state = TicketState::FailedProbe;
    }

    /// Transition to FAILED_SPECIALIST.
    pub fn fail_specialist(&mut self, reason: &str) {
        let transition =
            StateTransition::new(self.state, TicketState::FailedSpecialist, reason);
        self.history.push(transition);
        self.state = TicketState::FailedSpecialist;
    }

    /// Transition to NEED_CLARIFICATION.
    pub fn need_clarification(&mut self, question: &str) {
        let transition = StateTransition::new(
            self.state,
            TicketState::NeedClarification,
            &format!("Need clarification: {}", question),
        );
        self.history.push(transition);
        self.state = TicketState::NeedClarification;
    }

    /// Transition to ESCALATED.
    pub fn escalate(&mut self, reason: &str) {
        let transition = StateTransition::new(self.state, TicketState::Escalated, reason);
        self.history.push(transition);
        self.state = TicketState::Escalated;
    }

    /// Resume from NEED_CLARIFICATION to OPEN.
    pub fn resume(&mut self) {
        if self.state == TicketState::NeedClarification {
            let transition = StateTransition::new(
                self.state,
                TicketState::Open,
                "Clarification received, resuming",
            );
            self.history.push(transition);
            self.state = TicketState::Open;
        }
    }
}

/// Ticket stats with integrity guarantees.
#[derive(Debug, Clone, Default)]
pub struct TicketStats {
    /// Counts by state.
    counts: HashMap<TicketState, usize>,
    /// Total tickets processed.
    total: usize,
}

impl TicketStats {
    /// Create empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ticket in a state.
    pub fn record(&mut self, state: TicketState) {
        *self.counts.entry(state).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get count for a state.
    pub fn count(&self, state: TicketState) -> usize {
        self.counts.get(&state).copied().unwrap_or(0)
    }

    /// Get resolved count (RESOLVED state only).
    pub fn resolved(&self) -> usize {
        self.count(TicketState::Resolved)
    }

    /// Get failed count (FAILED_PROBE + FAILED_SPECIALIST).
    pub fn failed(&self) -> usize {
        self.count(TicketState::FailedProbe) + self.count(TicketState::FailedSpecialist)
    }

    /// Get total tickets.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Calculate success rate (resolved / total).
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.resolved() as f64 / self.total as f64
        }
    }

    /// Get summary for logging.
    pub fn summary(&self) -> StatsSummary {
        StatsSummary {
            total: self.total,
            resolved: self.resolved(),
            failed_probe: self.count(TicketState::FailedProbe),
            failed_specialist: self.count(TicketState::FailedSpecialist),
            need_clarification: self.count(TicketState::NeedClarification),
            escalated: self.count(TicketState::Escalated),
            success_rate: self.success_rate(),
        }
    }
}

/// Stats summary for display.
#[derive(Debug, Clone)]
pub struct StatsSummary {
    /// Total tickets.
    pub total: usize,
    /// Resolved count.
    pub resolved: usize,
    /// Failed probe count.
    pub failed_probe: usize,
    /// Failed specialist count.
    pub failed_specialist: usize,
    /// Need clarification count.
    pub need_clarification: usize,
    /// Escalated count.
    pub escalated: usize,
    /// Success rate (0.0-1.0).
    pub success_rate: f64,
}

impl StatsSummary {
    /// Format for logging.
    pub fn log_message(&self) -> String {
        format!(
            "[stats] total={} resolved={} failed_probe={} failed_specialist={} \
             need_clarification={} escalated={} success_rate={:.1}%",
            self.total,
            self.resolved,
            self.failed_probe,
            self.failed_specialist,
            self.need_clarification,
            self.escalated,
            self.success_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_state_labels() {
        assert_eq!(TicketState::Open.label(), "OPEN");
        assert_eq!(TicketState::Resolved.label(), "RESOLVED");
        assert_eq!(TicketState::FailedProbe.label(), "FAILED_PROBE");
    }

    #[test]
    fn test_ticket_state_properties() {
        assert!(!TicketState::Open.is_terminal());
        assert!(TicketState::Resolved.is_terminal());
        assert!(TicketState::Resolved.is_success());
        assert!(!TicketState::FailedProbe.is_success());
        assert!(TicketState::FailedProbe.is_failure());
    }

    #[test]
    fn test_resolution_criteria() {
        let criteria = ResolutionCriteria::new(true, true, 0.8);
        assert!(criteria.is_resolved());

        let low_confidence = ResolutionCriteria::new(true, true, 0.3);
        assert!(!low_confidence.is_resolved());
        assert_eq!(
            low_confidence.failure_reason(),
            Some("Confidence below threshold")
        );

        let no_answer = ResolutionCriteria::new(true, false, 0.9);
        assert!(!no_answer.is_resolved());
        assert_eq!(
            no_answer.failure_reason(),
            Some("No valid answer produced")
        );
    }

    #[test]
    fn test_state_machine_resolve() {
        let mut sm = TicketStateMachine::new("DSK-0101");
        assert_eq!(sm.state, TicketState::Open);

        let criteria = ResolutionCriteria::new(true, true, 0.85);
        sm.resolve(&criteria).unwrap();
        assert_eq!(sm.state, TicketState::Resolved);
        assert_eq!(sm.history.len(), 1);
    }

    #[test]
    fn test_state_machine_fail_probe() {
        let mut sm = TicketStateMachine::new("DSK-0101");
        sm.fail_probe(&["systemd_analyze", "free_h"]);
        assert_eq!(sm.state, TicketState::FailedProbe);
    }

    #[test]
    fn test_state_machine_fail_specialist() {
        let mut sm = TicketStateMachine::new("DSK-0101");
        sm.fail_specialist("Specialist timeout after 2 retries");
        assert_eq!(sm.state, TicketState::FailedSpecialist);
    }

    #[test]
    fn test_stats_tracking() {
        let mut stats = TicketStats::new();

        stats.record(TicketState::Resolved);
        stats.record(TicketState::Resolved);
        stats.record(TicketState::FailedProbe);
        stats.record(TicketState::FailedSpecialist);

        assert_eq!(stats.total(), 4);
        assert_eq!(stats.resolved(), 2);
        assert_eq!(stats.failed(), 2);
        assert!((stats.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stats_summary() {
        let mut stats = TicketStats::new();
        stats.record(TicketState::Resolved);
        stats.record(TicketState::FailedProbe);

        let summary = stats.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.resolved, 1);
        assert_eq!(summary.failed_probe, 1);
    }
}
