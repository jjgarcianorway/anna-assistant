//! Ticket lifecycle states and resolution types.

use serde::{Deserialize, Serialize};

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
