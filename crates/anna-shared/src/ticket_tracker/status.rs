//! Ticket status enum (v0.0.183).

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
            Self::Escalated => write!(f, "Escalated"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}
