//! Ticket lifecycle state machine (v0.0.433).
//!
//! Defines explicit states and transitions for tickets.

use super::contract::TicketOutcome;
use serde::{Deserialize, Serialize};

/// Ticket lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TicketState {
    /// Ticket just opened.
    Opened,
    /// Processing in progress.
    InProgress,
    /// Waiting for user clarification.
    AwaitingClarification,
    /// Successfully resolved.
    ResolvedSuccess,
    /// Partially resolved.
    ResolvedPartial,
    /// Failed to resolve.
    ResolvedFailure,
}

impl TicketState {
    /// Whether this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::ResolvedSuccess | Self::ResolvedPartial | Self::ResolvedFailure
        )
    }

    /// Whether this is a success state.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::ResolvedSuccess)
    }

    /// Whether this counts as resolved for stats.
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::ResolvedSuccess | Self::ResolvedPartial)
    }

    /// Whether this is a failure state.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::ResolvedFailure)
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Opened => "Opened",
            Self::InProgress => "In Progress",
            Self::AwaitingClarification => "Awaiting Clarification",
            Self::ResolvedSuccess => "Resolved (Success)",
            Self::ResolvedPartial => "Resolved (Partial)",
            Self::ResolvedFailure => "Failed",
        }
    }
}

impl Default for TicketState {
    fn default() -> Self {
        Self::Opened
    }
}

/// State transition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketTransition {
    /// Previous state.
    pub from: TicketState,
    /// New state.
    pub to: TicketState,
    /// Reason for transition.
    pub reason: String,
    /// Timestamp (unix ms).
    pub timestamp_ms: u64,
    /// Outcome that triggered this (if applicable).
    pub outcome: Option<TicketOutcome>,
}

impl TicketTransition {
    /// Create a new transition.
    pub fn new(from: TicketState, to: TicketState, reason: &str) -> Self {
        Self {
            from,
            to,
            reason: reason.to_string(),
            timestamp_ms: now_ms(),
            outcome: None,
        }
    }

    /// Create with outcome.
    pub fn from_outcome(from: TicketState, outcome: TicketOutcome) -> Self {
        let to = match outcome {
            TicketOutcome::Success => TicketState::ResolvedSuccess,
            TicketOutcome::Partial => TicketState::ResolvedPartial,
            TicketOutcome::ClarificationRequired => TicketState::AwaitingClarification,
            TicketOutcome::Unsupported
            | TicketOutcome::InternalError
            | TicketOutcome::Timeout
            | TicketOutcome::ParseError => TicketState::ResolvedFailure,
        };

        Self {
            from,
            to,
            reason: outcome.label().to_string(),
            timestamp_ms: now_ms(),
            outcome: Some(outcome),
        }
    }
}

/// Ticket lifecycle manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketLifecycle {
    /// Ticket ID.
    pub ticket_id: String,
    /// Current state.
    pub state: TicketState,
    /// State history.
    pub transitions: Vec<TicketTransition>,
    /// Original query.
    pub query: String,
    /// Created timestamp.
    pub created_ms: u64,
    /// Last update timestamp.
    pub updated_ms: u64,
    /// Final outcome (when terminal).
    pub final_outcome: Option<TicketOutcome>,
    /// Follow-up ticket IDs (for multi-turn).
    pub followups: Vec<String>,
    /// Parent ticket ID (if this is a follow-up).
    pub parent_id: Option<String>,
}

impl TicketLifecycle {
    /// Create a new ticket lifecycle.
    pub fn new(ticket_id: &str, query: &str) -> Self {
        let now = now_ms();
        Self {
            ticket_id: ticket_id.to_string(),
            state: TicketState::Opened,
            transitions: Vec::new(),
            query: query.to_string(),
            created_ms: now,
            updated_ms: now,
            final_outcome: None,
            followups: Vec::new(),
            parent_id: None,
        }
    }

    /// Create a follow-up ticket.
    pub fn create_followup(&mut self, followup_id: &str, query: &str) -> Self {
        self.followups.push(followup_id.to_string());

        let mut followup = Self::new(followup_id, query);
        followup.parent_id = Some(self.ticket_id.clone());
        followup
    }

    /// Start processing.
    pub fn start_processing(&mut self) -> Result<(), LifecycleError> {
        self.transition_to(TicketState::InProgress, "Processing started")
    }

    /// Apply an outcome.
    pub fn apply_outcome(&mut self, outcome: TicketOutcome) -> Result<(), LifecycleError> {
        let transition = TicketTransition::from_outcome(self.state, outcome);
        let new_state = transition.to;

        // Validate transition
        if !self.can_transition_to(new_state) {
            return Err(LifecycleError::InvalidTransition {
                from: self.state,
                to: new_state,
            });
        }

        self.state = new_state;
        self.updated_ms = now_ms();
        self.transitions.push(transition);

        if new_state.is_terminal() {
            self.final_outcome = Some(outcome);
        }

        Ok(())
    }

    /// Check if transition is valid.
    fn can_transition_to(&self, to: TicketState) -> bool {
        match (self.state, to) {
            // From Opened
            (TicketState::Opened, TicketState::InProgress) => true,

            // From InProgress
            (TicketState::InProgress, TicketState::AwaitingClarification) => true,
            (TicketState::InProgress, TicketState::ResolvedSuccess) => true,
            (TicketState::InProgress, TicketState::ResolvedPartial) => true,
            (TicketState::InProgress, TicketState::ResolvedFailure) => true,

            // From AwaitingClarification
            (TicketState::AwaitingClarification, TicketState::InProgress) => true,
            (TicketState::AwaitingClarification, TicketState::ResolvedSuccess) => true,
            (TicketState::AwaitingClarification, TicketState::ResolvedPartial) => true,
            (TicketState::AwaitingClarification, TicketState::ResolvedFailure) => true,

            // Terminal states cannot transition
            (TicketState::ResolvedSuccess, _) => false,
            (TicketState::ResolvedPartial, _) => false,
            (TicketState::ResolvedFailure, _) => false,

            _ => false,
        }
    }

    /// Generic transition.
    fn transition_to(&mut self, to: TicketState, reason: &str) -> Result<(), LifecycleError> {
        if !self.can_transition_to(to) {
            return Err(LifecycleError::InvalidTransition {
                from: self.state,
                to,
            });
        }

        let transition = TicketTransition::new(self.state, to, reason);
        self.state = to;
        self.updated_ms = now_ms();
        self.transitions.push(transition);
        Ok(())
    }

    /// Provide clarification (resume from awaiting).
    pub fn provide_clarification(&mut self, answer: &str) -> Result<(), LifecycleError> {
        if self.state != TicketState::AwaitingClarification {
            return Err(LifecycleError::NotAwaitingClarification);
        }
        self.transition_to(TicketState::InProgress, &format!("Clarification: {}", answer))
    }

    /// Get duration since created.
    pub fn duration_ms(&self) -> u64 {
        self.updated_ms.saturating_sub(self.created_ms)
    }

    /// Check if ticket is still active.
    pub fn is_active(&self) -> bool {
        !self.state.is_terminal()
    }

    /// Get state history as formatted string.
    pub fn format_history(&self) -> String {
        self.transitions
            .iter()
            .map(|t| format!("{} → {} ({})", t.from.label(), t.to.label(), t.reason))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Lifecycle errors.
#[derive(Debug, Clone)]
pub enum LifecycleError {
    /// Invalid state transition attempted.
    InvalidTransition { from: TicketState, to: TicketState },
    /// Tried to provide clarification when not waiting.
    NotAwaitingClarification,
    /// Ticket already resolved.
    AlreadyResolved,
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Cannot transition from {} to {}", from.label(), to.label())
            }
            Self::NotAwaitingClarification => {
                write!(f, "Ticket is not awaiting clarification")
            }
            Self::AlreadyResolved => {
                write!(f, "Ticket is already resolved")
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

/// Current time in milliseconds.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lifecycle() {
        let mut ticket = TicketLifecycle::new("DSK-001", "how much ram?");

        assert_eq!(ticket.state, TicketState::Opened);

        ticket.start_processing().unwrap();
        assert_eq!(ticket.state, TicketState::InProgress);

        ticket.apply_outcome(TicketOutcome::Success).unwrap();
        assert_eq!(ticket.state, TicketState::ResolvedSuccess);
        assert!(ticket.state.is_terminal());
    }

    #[test]
    fn test_clarification_flow() {
        let mut ticket = TicketLifecycle::new("DSK-002", "fix my service");

        ticket.start_processing().unwrap();
        ticket.apply_outcome(TicketOutcome::ClarificationRequired).unwrap();

        assert_eq!(ticket.state, TicketState::AwaitingClarification);
        assert!(!ticket.state.is_terminal());

        ticket.provide_clarification("nginx").unwrap();
        assert_eq!(ticket.state, TicketState::InProgress);

        ticket.apply_outcome(TicketOutcome::Success).unwrap();
        assert!(ticket.state.is_success());
    }

    #[test]
    fn test_failure_lifecycle() {
        let mut ticket = TicketLifecycle::new("DSK-003", "complex query");

        ticket.start_processing().unwrap();
        ticket.apply_outcome(TicketOutcome::Timeout).unwrap();

        assert_eq!(ticket.state, TicketState::ResolvedFailure);
        assert!(ticket.state.is_failure());
        assert_eq!(ticket.final_outcome, Some(TicketOutcome::Timeout));
    }

    #[test]
    fn test_invalid_transition() {
        let mut ticket = TicketLifecycle::new("DSK-004", "test");

        ticket.start_processing().unwrap();
        ticket.apply_outcome(TicketOutcome::Success).unwrap();

        // Cannot transition from terminal state
        let result = ticket.apply_outcome(TicketOutcome::Timeout);
        assert!(result.is_err());
    }

    #[test]
    fn test_followup_tickets() {
        let mut parent = TicketLifecycle::new("DSK-005", "original question");
        parent.start_processing().unwrap();
        parent.apply_outcome(TicketOutcome::ClarificationRequired).unwrap();

        let followup = parent.create_followup("DSK-005-1", "clarifying answer");

        assert_eq!(followup.parent_id, Some("DSK-005".to_string()));
        assert!(parent.followups.contains(&"DSK-005-1".to_string()));
    }
}
