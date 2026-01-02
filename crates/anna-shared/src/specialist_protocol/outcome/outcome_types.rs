//! Ticket outcome types and classification.

use serde::{Deserialize, Serialize};

/// Ticket outcome from user perspective
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// Full success: user got a complete, evidence-backed answer
    Success,
    /// Useful partial: user got meaningful but incomplete info
    UsefulPartial,
    /// Honest unknown: we admitted we don't know (still honest)
    HonestUnknown,
    /// Failed: user did not get useful information
    Failed,
    /// Internal error: parse/timeout that we couldn't recover from
    InternalError,
}

impl TicketOutcome {
    /// Check if this outcome counts as "resolved" for stats
    pub fn is_resolved(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::UsefulPartial | Self::HonestUnknown
        )
    }

    /// Check if this is a hard failure
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed | Self::InternalError)
    }

    /// Get XP value for this outcome
    pub fn xp_value(&self) -> i32 {
        match self {
            Self::Success => 10,
            Self::UsefulPartial => 6,
            Self::HonestUnknown => 3,
            Self::Failed => 0,
            Self::InternalError => -2,
        }
    }
}
