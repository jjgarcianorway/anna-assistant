//! Ticket outcome - semantic meaning of how a ticket was resolved

use serde::{Deserialize, Serialize};

/// v0.0.411: Ticket outcome - the semantic meaning of how a ticket was resolved
/// This is distinct from TicketState (lifecycle) vs TicketOutcome (quality/result)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// User got a correct, grounded answer
    Success,
    /// Some info provided, but limitations were explained
    Partial,
    /// Not enough evidence or too risky to answer
    CannotAnswerSafely,
    /// LLM response was invalid JSON or missing required fields
    ErrorParse,
    /// LLM or probe exceeded timeout
    ErrorTimeout,
    /// Probe or helper command failed
    ErrorTool,
    /// Unexpected internal failure (bug)
    ErrorInternal,
}

impl TicketOutcome {
    /// Check if this outcome counts as an error for stats
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ErrorParse | Self::ErrorTimeout | Self::ErrorTool | Self::ErrorInternal
        )
    }

    /// Check if this outcome counts as "resolved" (even if partial/cannot_answer)
    pub fn is_resolved(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::Partial | Self::CannotAnswerSafely
        )
    }

    /// Check if this is a full success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// XP value for this outcome (for RPG system)
    pub fn xp_value(&self) -> i32 {
        match self {
            Self::Success => 10,
            Self::Partial => 5,
            Self::CannotAnswerSafely => 2, // Honest but didn't solve
            Self::ErrorParse => 0,
            Self::ErrorTimeout => 0,
            Self::ErrorTool => 0,
            Self::ErrorInternal => -2, // Penalty for bugs
        }
    }
}

impl std::fmt::Display for TicketOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Partial => write!(f, "partial"),
            Self::CannotAnswerSafely => write!(f, "cannot_answer_safely"),
            Self::ErrorParse => write!(f, "error_parse"),
            Self::ErrorTimeout => write!(f, "error_timeout"),
            Self::ErrorTool => write!(f, "error_tool"),
            Self::ErrorInternal => write!(f, "error_internal"),
        }
    }
}
