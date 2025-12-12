//! Stats Integrity (Part F) - v0.0.436.
//!
//! Rewrite stats rules:
//! - A ticket is "resolved" only if ModelResultEnvelope.ok=true,
//!   OR non-LLM evidence-only answer delivered and confirmed
//! - If model output failed: "internal_failure"
//! - No more fake 100% success rates

use super::decoder::{DecodeError, DecodeResult};
use super::envelope::ModelResultEnvelope;
use serde::{Deserialize, Serialize};

/// Ticket outcome for stats tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// Successfully resolved (model ok=true or confirmed evidence answer).
    Resolved,
    /// Escalated to senior or human.
    Escalated,
    /// Internal failure (model error, timeout, parse failure).
    InternalFailure,
    /// User cancelled or abandoned.
    Cancelled,
    /// Still in progress.
    InProgress,
}

impl TicketOutcome {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Escalated => "escalated",
            Self::InternalFailure => "internal_failure",
            Self::Cancelled => "cancelled",
            Self::InProgress => "in_progress",
        }
    }

    /// Check if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::InProgress)
    }

    /// Check if this counts as success.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Resolved)
    }
}

/// Stats for a time period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeriodStats {
    /// Total tickets in period.
    pub total_tickets: u64,
    /// Successfully resolved.
    pub resolved: u64,
    /// Escalated to senior/human.
    pub escalated: u64,
    /// Internal failures (model errors).
    pub internal_failures: u64,
    /// User cancelled.
    pub cancelled: u64,
    /// Average response time in milliseconds.
    pub avg_response_ms: u64,
    /// Total response time (for calculating average).
    #[serde(skip)]
    pub total_response_ms: u64,
}

impl PeriodStats {
    /// Create new empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ticket outcome.
    pub fn record(&mut self, outcome: TicketOutcome, response_ms: u64) {
        self.total_tickets += 1;
        self.total_response_ms += response_ms;

        match outcome {
            TicketOutcome::Resolved => self.resolved += 1,
            TicketOutcome::Escalated => self.escalated += 1,
            TicketOutcome::InternalFailure => self.internal_failures += 1,
            TicketOutcome::Cancelled => self.cancelled += 1,
            TicketOutcome::InProgress => {
                // Don't count in-progress as terminal
                self.total_tickets -= 1;
                self.total_response_ms -= response_ms;
            }
        }

        self.update_average();
    }

    /// Update the average response time.
    fn update_average(&mut self) {
        let terminal = self.resolved + self.escalated + self.internal_failures + self.cancelled;
        if terminal > 0 {
            self.avg_response_ms = self.total_response_ms / terminal;
        }
    }

    /// Get resolution rate (0.0 to 1.0).
    pub fn resolution_rate(&self) -> f64 {
        let terminal = self.resolved + self.escalated + self.internal_failures + self.cancelled;
        if terminal == 0 {
            return 0.0;
        }
        self.resolved as f64 / terminal as f64
    }

    /// Get escalation rate (0.0 to 1.0).
    pub fn escalation_rate(&self) -> f64 {
        let terminal = self.resolved + self.escalated + self.internal_failures + self.cancelled;
        if terminal == 0 {
            return 0.0;
        }
        self.escalated as f64 / terminal as f64
    }

    /// Get internal failure rate (0.0 to 1.0).
    pub fn failure_rate(&self) -> f64 {
        let terminal = self.resolved + self.escalated + self.internal_failures + self.cancelled;
        if terminal == 0 {
            return 0.0;
        }
        self.internal_failures as f64 / terminal as f64
    }

    /// Merge another period's stats.
    pub fn merge(&mut self, other: &PeriodStats) {
        self.total_tickets += other.total_tickets;
        self.resolved += other.resolved;
        self.escalated += other.escalated;
        self.internal_failures += other.internal_failures;
        self.cancelled += other.cancelled;
        self.total_response_ms += other.total_response_ms;
        self.update_average();
    }
}

/// Determine ticket outcome from decode result.
pub fn outcome_from_decode(result: &DecodeResult) -> TicketOutcome {
    match result {
        DecodeResult::Success(envelope) => outcome_from_envelope(envelope),
        DecodeResult::Failed(error) => outcome_from_error(error),
    }
}

/// Determine ticket outcome from envelope.
pub fn outcome_from_envelope(envelope: &ModelResultEnvelope) -> TicketOutcome {
    if envelope.ok {
        TicketOutcome::Resolved
    } else {
        // Model returned ok=false - this is escalation or need clarification
        TicketOutcome::Escalated
    }
}

/// Determine ticket outcome from decode error.
pub fn outcome_from_error(error: &DecodeError) -> TicketOutcome {
    // All decode errors are internal failures
    match error {
        DecodeError::ModelTimeout { .. } => TicketOutcome::InternalFailure,
        DecodeError::NoFrame { .. } => TicketOutcome::InternalFailure,
        DecodeError::IncompleteFrame { .. } => TicketOutcome::InternalFailure,
        DecodeError::MultipleFrames => TicketOutcome::InternalFailure,
        DecodeError::JsonParseError { .. } => TicketOutcome::InternalFailure,
        DecodeError::EnvelopeInvalid { .. } => TicketOutcome::InternalFailure,
        DecodeError::EmptyOutput => TicketOutcome::InternalFailure,
    }
}

/// Stats summary for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    /// Period label (e.g., "today", "this_week").
    pub period: String,
    /// Stats for the period.
    pub stats: PeriodStats,
    /// Resolution rate as percentage string.
    pub resolution_pct: String,
    /// Failure rate as percentage string.
    pub failure_pct: String,
}

impl StatsSummary {
    /// Create a summary from period stats.
    pub fn new(period: &str, stats: PeriodStats) -> Self {
        let resolution_pct = format!("{:.1}%", stats.resolution_rate() * 100.0);
        let failure_pct = format!("{:.1}%", stats.failure_rate() * 100.0);

        Self {
            period: period.to_string(),
            stats,
            resolution_pct,
            failure_pct,
        }
    }

    /// Format as compact display string.
    pub fn display(&self) -> String {
        format!(
            "{}: {} tickets | {} resolved | {} internal failures | avg {}ms",
            self.period,
            self.stats.total_tickets,
            self.resolution_pct,
            self.failure_pct,
            self.stats.avg_response_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anna_proto::envelope::ModelRole;

    #[test]
    fn test_ticket_outcome() {
        assert!(TicketOutcome::Resolved.is_success());
        assert!(TicketOutcome::Resolved.is_terminal());
        assert!(!TicketOutcome::InProgress.is_terminal());
        assert!(!TicketOutcome::InternalFailure.is_success());
    }

    #[test]
    fn test_period_stats_record() {
        let mut stats = PeriodStats::new();

        stats.record(TicketOutcome::Resolved, 100);
        stats.record(TicketOutcome::Resolved, 200);
        stats.record(TicketOutcome::InternalFailure, 150);

        assert_eq!(stats.total_tickets, 3);
        assert_eq!(stats.resolved, 2);
        assert_eq!(stats.internal_failures, 1);
        assert_eq!(stats.avg_response_ms, 150); // (100+200+150)/3
    }

    #[test]
    fn test_resolution_rate() {
        let mut stats = PeriodStats::new();

        stats.record(TicketOutcome::Resolved, 100);
        stats.record(TicketOutcome::Resolved, 100);
        stats.record(TicketOutcome::InternalFailure, 100);
        stats.record(TicketOutcome::Escalated, 100);

        // 2 resolved out of 4 terminal = 50%
        assert!((stats.resolution_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_outcome_from_envelope() {
        let success = ModelResultEnvelope::success(
            ModelRole::Junior,
            "DSK-001",
            "Test",
            0.9,
        );
        assert_eq!(outcome_from_envelope(&success), TicketOutcome::Resolved);

        let failure = ModelResultEnvelope::failure(
            ModelRole::Junior,
            "DSK-001",
            vec![],
        );
        assert_eq!(outcome_from_envelope(&failure), TicketOutcome::Escalated);
    }

    #[test]
    fn test_outcome_from_error() {
        let timeout = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };
        assert_eq!(outcome_from_error(&timeout), TicketOutcome::InternalFailure);

        let no_frame = DecodeError::NoFrame {
            raw_output: "test".to_string(),
        };
        assert_eq!(outcome_from_error(&no_frame), TicketOutcome::InternalFailure);
    }

    #[test]
    fn test_stats_merge() {
        let mut stats1 = PeriodStats::new();
        stats1.record(TicketOutcome::Resolved, 100);

        let mut stats2 = PeriodStats::new();
        stats2.record(TicketOutcome::InternalFailure, 200);

        stats1.merge(&stats2);

        assert_eq!(stats1.total_tickets, 2);
        assert_eq!(stats1.resolved, 1);
        assert_eq!(stats1.internal_failures, 1);
    }

    #[test]
    fn test_stats_summary() {
        let mut stats = PeriodStats::new();
        stats.record(TicketOutcome::Resolved, 100);
        stats.record(TicketOutcome::Resolved, 100);
        stats.record(TicketOutcome::InternalFailure, 100);

        let summary = StatsSummary::new("today", stats);

        assert!(summary.resolution_pct.contains("66"));
        assert!(summary.failure_pct.contains("33"));
    }

    #[test]
    fn test_in_progress_not_counted() {
        let mut stats = PeriodStats::new();
        stats.record(TicketOutcome::Resolved, 100);
        stats.record(TicketOutcome::InProgress, 50);

        // In-progress should not be counted
        assert_eq!(stats.total_tickets, 1);
        assert_eq!(stats.resolved, 1);
    }
}
