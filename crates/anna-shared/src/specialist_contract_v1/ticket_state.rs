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

// Re-export types from sibling modules
pub use super::ticket_resolution::{
    ResolutionCriteria, StateTransition, MIN_CONFIDENCE_FOR_RESOLVED,
};
pub use super::ticket_state_machine::TicketStateMachine;
pub use super::ticket_stats::{StatsSummary, TicketStats};

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
}
