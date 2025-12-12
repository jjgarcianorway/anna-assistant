//! Strict ticket lifecycle state machine (v0.0.426).
//!
//! This module enforces a finite state machine for each ticket with:
//! - Strict state transitions (no skipping states)
//! - Clear "resolved" vs "failed" semantics
//! - Connection to specialist JSON outcomes
//! - Honest metrics based on actual outcomes

use crate::specialist_v3::{ResponseStatus, SpecialistResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strict ticket state - finite state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketLifecycleState {
    /// Created, not yet dispatched
    New,
    /// Assigned to specialist, probes running
    InProgress,
    /// Specialist produced a response (any status)
    Answered,
    /// User got a coherent answer (even "I don't know")
    UserSatisfied,
    /// Hard internal failure prevented answer
    Failed,
    /// User aborted or ticket invalid
    Cancelled,
}

impl Default for TicketLifecycleState {
    fn default() -> Self {
        Self::New
    }
}

impl std::fmt::Display for TicketLifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Answered => write!(f, "answered"),
            Self::UserSatisfied => write!(f, "user_satisfied"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Final outcome classification for stats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketResolution {
    /// Full success with grounded answer
    ResolvedSuccess,
    /// Partial answer with limitations
    ResolvedPartial,
    /// Honest "I don't know" delivered
    ResolvedHonestUnknown,
    /// Question outside domain, routed
    ResolvedUnsupported,
    /// Hard failure (parse, crash, timeout)
    Failed,
    /// User cancelled
    Cancelled,
    /// Still in progress
    Pending,
}

impl TicketResolution {
    /// Check if this counts as "resolved" for stats
    pub fn is_resolved(&self) -> bool {
        matches!(
            self,
            Self::ResolvedSuccess
                | Self::ResolvedPartial
                | Self::ResolvedHonestUnknown
                | Self::ResolvedUnsupported
        )
    }

    /// Check if this is a full success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::ResolvedSuccess)
    }

    /// Check if this is a failure
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// XP value for this resolution
    pub fn xp_value(&self) -> i32 {
        match self {
            Self::ResolvedSuccess => 10,
            Self::ResolvedPartial => 6,
            Self::ResolvedHonestUnknown | Self::ResolvedUnsupported => 3,
            Self::Failed => 0,
            Self::Cancelled => 0,
            Self::Pending => 0,
        }
    }
}

impl std::fmt::Display for TicketResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResolvedSuccess => write!(f, "resolved_success"),
            Self::ResolvedPartial => write!(f, "resolved_partial"),
            Self::ResolvedHonestUnknown => write!(f, "resolved_honest_unknown"),
            Self::ResolvedUnsupported => write!(f, "resolved_unsupported"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

/// Internal error classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InternalError {
    /// JSON parsing failed after retries
    ParseError { attempts: u8, last_error: String },
    /// LLM timeout
    Timeout { timeout_ms: u64 },
    /// Probe execution failed
    ProbeFailure { probe_id: String, error: String },
    /// Unexpected crash/panic
    InternalCrash { context: String },
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError { attempts, .. } => write!(f, "parse_error (attempts: {})", attempts),
            Self::Timeout { timeout_ms } => write!(f, "timeout ({}ms)", timeout_ms),
            Self::ProbeFailure { probe_id, .. } => write!(f, "probe_failure ({})", probe_id),
            Self::InternalCrash { .. } => write!(f, "internal_crash"),
        }
    }
}

/// Complete ticket record with lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketRecord {
    /// Unique ticket ID
    pub ticket_id: String,
    /// Creation timestamp (Unix millis)
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Current lifecycle state
    pub state: TicketLifecycleState,
    /// Original user question
    pub user_question: String,
    /// Final specialist status (from JSON)
    #[serde(default)]
    pub final_specialist_status: Option<ResponseStatus>,
    /// Final confidence (0.0-1.0)
    pub final_confidence: f32,
    /// Final severity
    #[serde(default)]
    pub final_severity: Option<String>,
    /// Whether ticket was escalated
    pub escalated: bool,
    /// Escalation chain (e.g., ["desktop.junior", "desktop.senior"])
    #[serde(default)]
    pub escalation_chain: Vec<String>,
    /// Total latency in milliseconds
    pub latency_ms: u64,
    /// Internal error (if state == Failed)
    #[serde(default)]
    pub internal_error: Option<InternalError>,
    /// Final answer delivered to user
    #[serde(default)]
    pub final_answer: Option<String>,
    /// v0.0.426: Is this a legacy ticket (pre-strict lifecycle)?
    #[serde(default)]
    pub is_legacy: bool,
    /// State transition history
    #[serde(default)]
    pub transitions: Vec<StateTransition>,
}

/// State transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: TicketLifecycleState,
    pub to: TicketLifecycleState,
    pub at: u64,
    pub reason: Option<String>,
}

impl TicketRecord {
    /// Create a new ticket
    pub fn new(ticket_id: impl Into<String>, question: impl Into<String>) -> Self {
        let now = current_millis();
        Self {
            ticket_id: ticket_id.into(),
            created_at: now,
            updated_at: now,
            state: TicketLifecycleState::New,
            user_question: question.into(),
            final_specialist_status: None,
            final_confidence: 0.0,
            final_severity: None,
            escalated: false,
            escalation_chain: vec![],
            latency_ms: 0,
            internal_error: None,
            final_answer: None,
            is_legacy: false,
            transitions: vec![],
        }
    }

    /// Transition to a new state (with validation)
    pub fn transition(
        &mut self,
        to: TicketLifecycleState,
        reason: Option<String>,
    ) -> Result<(), String> {
        if !self.can_transition_to(to) {
            return Err(format!(
                "Invalid transition: {} -> {} (ticket: {})",
                self.state, to, self.ticket_id
            ));
        }

        let transition = StateTransition {
            from: self.state,
            to,
            at: current_millis(),
            reason,
        };
        self.transitions.push(transition);
        self.state = to;
        self.updated_at = current_millis();
        Ok(())
    }

    /// Check if transition is valid
    fn can_transition_to(&self, to: TicketLifecycleState) -> bool {
        use TicketLifecycleState::*;
        match (self.state, to) {
            // Forward transitions
            (New, InProgress) => true,
            (InProgress, Answered) => true,
            (Answered, UserSatisfied) => true,
            (Answered, Failed) => true,
            // Any state can be cancelled
            (_, Cancelled) => true,
            // InProgress can fail directly (e.g., probe failure)
            (InProgress, Failed) => true,
            // Already terminal
            (UserSatisfied | Failed | Cancelled, _) => false,
            // Invalid transition
            _ => false,
        }
    }

    /// Move to in_progress (assigned to specialist)
    pub fn start_processing(&mut self, specialist: &str) -> Result<(), String> {
        self.escalation_chain.push(specialist.to_string());
        self.transition(
            TicketLifecycleState::InProgress,
            Some(format!("Assigned to {}", specialist)),
        )
    }

    /// Mark as answered with specialist response
    pub fn mark_answered(&mut self, response: &SpecialistResponse) -> Result<(), String> {
        self.final_specialist_status = Some(response.status);
        self.final_confidence = response.confidence;
        self.final_severity = Some(format!("{:?}", response.severity).to_lowercase());
        self.transition(
            TicketLifecycleState::Answered,
            Some(format!("Specialist status: {:?}", response.status)),
        )
    }

    /// Mark as user satisfied (answer delivered)
    pub fn mark_user_satisfied(&mut self, answer: &str) -> Result<(), String> {
        self.final_answer = Some(answer.to_string());
        self.latency_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(
            TicketLifecycleState::UserSatisfied,
            Some("Answer delivered".to_string()),
        )
    }

    /// Mark as failed with internal error
    pub fn mark_failed(&mut self, error: InternalError) -> Result<(), String> {
        self.internal_error = Some(error.clone());
        self.latency_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(
            TicketLifecycleState::Failed,
            Some(format!("Error: {}", error)),
        )
    }

    /// Mark as cancelled
    pub fn mark_cancelled(&mut self, reason: &str) -> Result<(), String> {
        self.latency_ms = self.updated_at.saturating_sub(self.created_at);
        self.transition(TicketLifecycleState::Cancelled, Some(reason.to_string()))
    }

    /// Escalate to another specialist
    pub fn escalate_to(&mut self, specialist: &str) {
        self.escalated = true;
        self.escalation_chain.push(specialist.to_string());
        self.updated_at = current_millis();
    }

    /// Get the final resolution classification
    pub fn resolution(&self) -> TicketResolution {
        match self.state {
            TicketLifecycleState::UserSatisfied => match self.final_specialist_status {
                Some(ResponseStatus::Success) => TicketResolution::ResolvedSuccess,
                Some(ResponseStatus::Partial) => TicketResolution::ResolvedPartial,
                Some(ResponseStatus::NoData) => TicketResolution::ResolvedHonestUnknown,
                Some(ResponseStatus::Unsupported) => TicketResolution::ResolvedUnsupported,
                Some(ResponseStatus::Error) => TicketResolution::Failed,
                None => TicketResolution::ResolvedHonestUnknown,
            },
            TicketLifecycleState::Failed => TicketResolution::Failed,
            TicketLifecycleState::Cancelled => TicketResolution::Cancelled,
            _ => TicketResolution::Pending,
        }
    }

    /// Check if ticket is in terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            TicketLifecycleState::UserSatisfied
                | TicketLifecycleState::Failed
                | TicketLifecycleState::Cancelled
        )
    }

    /// Get the lead specialist (last in chain)
    pub fn lead_specialist(&self) -> Option<&str> {
        self.escalation_chain.last().map(|s| s.as_str())
    }
}

/// Honest reliability metrics computed from ticket records
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    /// Total tickets (all created)
    pub total_tickets: usize,
    /// Resolved with full success
    pub resolved_success: usize,
    /// Resolved with partial answer
    pub resolved_partial: usize,
    /// Resolved with honest "I don't know"
    pub honest_unknown: usize,
    /// Resolved with "unsupported" routing
    pub resolved_unsupported: usize,
    /// Failed (hard internal failures)
    pub failed: usize,
    /// Cancelled by user
    pub cancelled: usize,
    /// Escalated tickets
    pub escalated: usize,
    /// Parse errors specifically
    pub parse_errors: usize,
    /// Timeout errors specifically
    pub timeout_errors: usize,
    /// Internal errors (crashes, etc)
    pub internal_errors: usize,
    /// Average response time (ms)
    pub avg_response_ms: u64,
    /// Success rate: resolved_success / total_tickets
    pub success_rate: f32,
    /// Reliability rate: resolved_success / (resolved_success + failed + parse_errors + internal_errors)
    pub reliability_rate: f32,
}

impl ReliabilityMetrics {
    /// Compute metrics from ticket records
    pub fn compute(tickets: &[TicketRecord]) -> Self {
        let mut metrics = Self::default();

        if tickets.is_empty() {
            return metrics;
        }

        let mut total_latency = 0u64;
        let mut latency_count = 0usize;

        for ticket in tickets {
            // Skip legacy tickets from strict metrics
            if ticket.is_legacy {
                continue;
            }

            metrics.total_tickets += 1;

            let resolution = ticket.resolution();
            match resolution {
                TicketResolution::ResolvedSuccess => metrics.resolved_success += 1,
                TicketResolution::ResolvedPartial => metrics.resolved_partial += 1,
                TicketResolution::ResolvedHonestUnknown => metrics.honest_unknown += 1,
                TicketResolution::ResolvedUnsupported => metrics.resolved_unsupported += 1,
                TicketResolution::Failed => metrics.failed += 1,
                TicketResolution::Cancelled => metrics.cancelled += 1,
                TicketResolution::Pending => {} // Not counted
            }

            // Count error types
            if let Some(ref error) = ticket.internal_error {
                match error {
                    InternalError::ParseError { .. } => metrics.parse_errors += 1,
                    InternalError::Timeout { .. } => metrics.timeout_errors += 1,
                    InternalError::ProbeFailure { .. } | InternalError::InternalCrash { .. } => {
                        metrics.internal_errors += 1;
                    }
                }
            }

            if ticket.escalated {
                metrics.escalated += 1;
            }

            // Latency for terminal tickets
            if ticket.is_terminal() && ticket.latency_ms > 0 {
                total_latency += ticket.latency_ms;
                latency_count += 1;
            }
        }

        // Calculate averages
        if latency_count > 0 {
            metrics.avg_response_ms = total_latency / latency_count as u64;
        }

        // Calculate rates
        if metrics.total_tickets > 0 {
            metrics.success_rate =
                (metrics.resolved_success as f32 / metrics.total_tickets as f32) * 100.0;

            // Reliability rate: success / (success + all failures)
            let failure_pool = metrics.resolved_success
                + metrics.failed
                + metrics.parse_errors
                + metrics.internal_errors;
            if failure_pool > 0 {
                metrics.reliability_rate =
                    (metrics.resolved_success as f32 / failure_pool as f32) * 100.0;
            }
        }

        metrics
    }
}

impl std::fmt::Display for ReliabilityMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[service desk]")?;
        writeln!(f, "  total_tickets         {}", self.total_tickets)?;
        writeln!(f, "  resolved_success      {}", self.resolved_success)?;
        writeln!(f, "  resolved_partial      {}", self.resolved_partial)?;
        writeln!(f, "  honest_unknown        {}", self.honest_unknown)?;
        writeln!(f, "  failed                {}", self.failed)?;
        writeln!(f, "  escalated             {}", self.escalated)?;
        writeln!(
            f,
            "  avg_response          {:.1}s",
            self.avg_response_ms as f64 / 1000.0
        )?;
        writeln!(f)?;
        writeln!(f, "[reliability]")?;
        writeln!(f, "  success_rate          {:.0}%", self.success_rate)?;
        writeln!(f, "  reliability_rate      {:.0}%", self.reliability_rate)?;
        writeln!(f, "  parse_errors          {}", self.parse_errors)?;
        writeln!(f, "  timeout_errors        {}", self.timeout_errors)?;
        writeln!(f, "  internal_errors       {}", self.internal_errors)?;
        Ok(())
    }
}

/// Per-specialist metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistMetrics {
    /// Specialist identifier (e.g., "desktop.junior")
    pub specialist_id: String,
    /// All tickets where specialist appears in chain
    pub tickets_handled: usize,
    /// Tickets where specialist is lead (final in chain)
    pub tickets_lead: usize,
    /// Success count (as lead)
    pub success_count: usize,
    /// Partial count (as lead)
    pub partial_count: usize,
    /// Honest unknown count (as lead)
    pub honest_unknown_count: usize,
    /// Failed count (as lead)
    pub failed_count: usize,
    /// Total XP earned
    pub xp: i64,
    /// Success rate as lead
    pub success_rate: f32,
}

impl SpecialistMetrics {
    /// Get title based on XP and success rate
    pub fn title(&self) -> &'static str {
        // Require minimum success rate for promotion
        let rate = self.success_rate / 100.0;

        if self.xp >= 2000 && rate >= 0.85 {
            "Senior"
        } else if self.xp >= 500 && rate >= 0.70 {
            "Proficient"
        } else {
            "Apprentice"
        }
    }
}

/// Compute per-specialist metrics from ticket records
pub fn compute_specialist_metrics(tickets: &[TicketRecord]) -> HashMap<String, SpecialistMetrics> {
    let mut metrics: HashMap<String, SpecialistMetrics> = HashMap::new();

    for ticket in tickets {
        if ticket.is_legacy {
            continue;
        }

        let resolution = ticket.resolution();

        // Track all specialists in chain
        for specialist in &ticket.escalation_chain {
            let m = metrics
                .entry(specialist.clone())
                .or_insert_with(|| SpecialistMetrics {
                    specialist_id: specialist.clone(),
                    ..Default::default()
                });
            m.tickets_handled += 1;
        }

        // Track lead specialist
        if let Some(lead) = ticket.lead_specialist() {
            let m = metrics
                .entry(lead.to_string())
                .or_insert_with(|| SpecialistMetrics {
                    specialist_id: lead.to_string(),
                    ..Default::default()
                });
            m.tickets_lead += 1;

            // Count by resolution
            match resolution {
                TicketResolution::ResolvedSuccess => {
                    m.success_count += 1;
                    m.xp += resolution.xp_value() as i64;
                }
                TicketResolution::ResolvedPartial => {
                    m.partial_count += 1;
                    m.xp += resolution.xp_value() as i64;
                }
                TicketResolution::ResolvedHonestUnknown | TicketResolution::ResolvedUnsupported => {
                    m.honest_unknown_count += 1;
                    m.xp += resolution.xp_value() as i64;
                }
                TicketResolution::Failed => {
                    m.failed_count += 1;
                    // No XP for failures
                }
                _ => {}
            }
        }
    }

    // Calculate success rates
    for m in metrics.values_mut() {
        if m.tickets_lead > 0 {
            m.success_rate = (m.success_count as f32 / m.tickets_lead as f32) * 100.0;
        }
    }

    metrics
}

/// Format specialist roster for display
pub fn format_specialist_roster(metrics: &HashMap<String, SpecialistMetrics>) -> String {
    let mut output = String::new();
    output.push_str("[staff roster]\n");

    // Group by department
    let mut by_dept: HashMap<String, Vec<&SpecialistMetrics>> = HashMap::new();
    for m in metrics.values() {
        let dept = m
            .specialist_id
            .split('.')
            .next()
            .unwrap_or("unknown")
            .to_uppercase();
        by_dept.entry(dept).or_default().push(m);
    }

    // Sort departments
    let mut depts: Vec<_> = by_dept.keys().collect();
    depts.sort();

    for dept in depts {
        output.push_str(&format!("  {}\n", dept));
        if let Some(staff) = by_dept.get(dept) {
            let mut sorted_staff: Vec<_> = staff.iter().collect();
            sorted_staff.sort_by(|a, b| b.xp.cmp(&a.xp));

            for m in sorted_staff {
                let name = m
                    .specialist_id
                    .split('.')
                    .last()
                    .unwrap_or(&m.specialist_id);
                let title = m.title();
                output.push_str(&format!(
                    "    {} ({})    tickets: {:3}   lead: {:3}   success: {:3}   failed: {:2}   honest_unknown: {:2}   rate: {:3.0}%   {}\n",
                    name, if m.specialist_id.contains("senior") { "Sr" } else { "Jr" },
                    m.tickets_handled, m.tickets_lead, m.success_count, m.failed_count,
                    m.honest_unknown_count, m.success_rate, title
                ));
            }
        }
    }

    output
}

/// Get current time in milliseconds
fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_lifecycle_success() {
        let mut ticket = TicketRecord::new("DSK-001", "Why is my boot slow?");
        assert_eq!(ticket.state, TicketLifecycleState::New);

        ticket.start_processing("desktop.junior").unwrap();
        assert_eq!(ticket.state, TicketLifecycleState::InProgress);

        let response = SpecialistResponse::success("DSK-001", "Boot time is normal at 15s");
        ticket.mark_answered(&response).unwrap();
        assert_eq!(ticket.state, TicketLifecycleState::Answered);

        ticket
            .mark_user_satisfied("Your boot time is normal")
            .unwrap();
        assert_eq!(ticket.state, TicketLifecycleState::UserSatisfied);
        assert_eq!(ticket.resolution(), TicketResolution::ResolvedSuccess);
    }

    #[test]
    fn test_ticket_lifecycle_failure() {
        let mut ticket = TicketRecord::new("DSK-002", "Test query");
        ticket.start_processing("desktop.junior").unwrap();

        let error = InternalError::ParseError {
            attempts: 2,
            last_error: "Invalid JSON".to_string(),
        };
        ticket.mark_failed(error).unwrap();

        assert_eq!(ticket.state, TicketLifecycleState::Failed);
        assert_eq!(ticket.resolution(), TicketResolution::Failed);
        assert!(ticket.internal_error.is_some());
    }

    #[test]
    fn test_ticket_lifecycle_honest_unknown() {
        let mut ticket = TicketRecord::new("DSK-003", "Unknown topic");
        ticket.start_processing("desktop.junior").unwrap();

        let response = SpecialistResponse::no_data("DSK-003", "No data available");
        ticket.mark_answered(&response).unwrap();
        ticket
            .mark_user_satisfied("I couldn't find information about this")
            .unwrap();

        assert_eq!(ticket.resolution(), TicketResolution::ResolvedHonestUnknown);
    }

    #[test]
    fn test_invalid_transition() {
        let mut ticket = TicketRecord::new("DSK-004", "Test");
        // Can't go directly to Answered from New
        let result = ticket.transition(TicketLifecycleState::Answered, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reliability_metrics() {
        let tickets = vec![
            create_success_ticket("T1"),
            create_success_ticket("T2"),
            create_partial_ticket("T3"),
            create_failed_ticket("T4"),
            create_honest_unknown_ticket("T5"),
        ];

        let metrics = ReliabilityMetrics::compute(&tickets);
        assert_eq!(metrics.total_tickets, 5);
        assert_eq!(metrics.resolved_success, 2);
        assert_eq!(metrics.resolved_partial, 1);
        assert_eq!(metrics.honest_unknown, 1);
        assert_eq!(metrics.failed, 1);
        assert_eq!(metrics.success_rate, 40.0); // 2/5
    }

    #[test]
    fn test_specialist_metrics() {
        let tickets = vec![
            create_success_ticket_with_specialist("T1", "desktop.junior"),
            create_success_ticket_with_specialist("T2", "desktop.junior"),
            create_failed_ticket_with_specialist("T3", "desktop.junior"),
            create_success_ticket_with_specialist("T4", "desktop.senior"),
        ];

        let metrics = compute_specialist_metrics(&tickets);

        let junior = metrics.get("desktop.junior").unwrap();
        assert_eq!(junior.tickets_lead, 3);
        assert_eq!(junior.success_count, 2);
        assert_eq!(junior.failed_count, 1);
        assert!((junior.success_rate - 66.67).abs() < 1.0);

        let senior = metrics.get("desktop.senior").unwrap();
        assert_eq!(senior.tickets_lead, 1);
        assert_eq!(senior.success_count, 1);
        assert_eq!(senior.success_rate, 100.0);
    }

    #[test]
    fn test_xp_and_title() {
        let mut m = SpecialistMetrics {
            specialist_id: "test".to_string(),
            tickets_lead: 100,
            success_count: 90,
            xp: 2000,
            success_rate: 90.0,
            ..Default::default()
        };
        assert_eq!(m.title(), "Senior");

        m.xp = 500;
        m.success_rate = 75.0;
        assert_eq!(m.title(), "Proficient");

        m.xp = 100;
        m.success_rate = 50.0;
        assert_eq!(m.title(), "Apprentice");
    }

    // Helper functions for tests
    fn create_success_ticket(id: &str) -> TicketRecord {
        create_success_ticket_with_specialist(id, "desktop.junior")
    }

    fn create_success_ticket_with_specialist(id: &str, specialist: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing(specialist).unwrap();
        let response = SpecialistResponse::success(id, "Success");
        ticket.mark_answered(&response).unwrap();
        ticket.mark_user_satisfied("Answer").unwrap();
        ticket
    }

    fn create_partial_ticket(id: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing("desktop.junior").unwrap();
        let response = SpecialistResponse::partial(id, "Partial");
        ticket.mark_answered(&response).unwrap();
        ticket.mark_user_satisfied("Partial answer").unwrap();
        ticket
    }

    fn create_failed_ticket(id: &str) -> TicketRecord {
        create_failed_ticket_with_specialist(id, "desktop.junior")
    }

    fn create_failed_ticket_with_specialist(id: &str, specialist: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing(specialist).unwrap();
        ticket
            .mark_failed(InternalError::ParseError {
                attempts: 2,
                last_error: "Invalid JSON".to_string(),
            })
            .unwrap();
        ticket
    }

    fn create_honest_unknown_ticket(id: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new(id, "Test");
        ticket.start_processing("desktop.junior").unwrap();
        let response = SpecialistResponse::no_data(id, "No data");
        ticket.mark_answered(&response).unwrap();
        ticket.mark_user_satisfied("I don't know").unwrap();
        ticket
    }
}
