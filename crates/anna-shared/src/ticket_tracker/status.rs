//! Ticket status enum (v0.0.258).
//!
//! v0.0.258: Added PendingRetry status for tickets that can be retried during idle time.

use serde::{Deserialize, Serialize};

/// Ticket status in the Service Desk workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    /// Just created, awaiting triage
    New,
    /// Assigned to a team member
    Assigned,
    /// Being actively worked on
    InProgress,
    /// Waiting for user input
    PendingUser,
    /// v0.0.258: Waiting for retry during idle time (low confidence answer)
    PendingRetry,
    /// Escalated to senior
    Escalated,
    /// Successfully resolved
    Resolved,
    /// Closed without resolution
    Closed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::Assigned => write!(f, "Assigned"),
            Self::InProgress => write!(f, "In Progress"),
            Self::PendingUser => write!(f, "Pending User"),
            Self::PendingRetry => write!(f, "Pending Retry"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}

impl TicketStatus {
    /// Check if ticket is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Resolved | Self::Closed)
    }

    /// Check if ticket can be retried
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::PendingRetry)
    }

    /// Check if ticket is waiting for something
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::PendingUser | Self::PendingRetry)
    }
}
