//! Outcome enums for execution trace (v0.0.184).

use serde::{Deserialize, Serialize};

/// Outcome of the specialist stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistOutcome {
    /// Specialist LLM produced an answer
    Ok,
    /// Specialist LLM timed out
    Timeout,
    /// Specialist exceeded budget before completing
    BudgetExceeded,
    /// Specialist was skipped (deterministic route answered directly)
    Skipped,
    /// Specialist returned an error
    Error,
}

impl std::fmt::Display for SpecialistOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Ok => "ok",
            Self::Timeout => "timeout",
            Self::BudgetExceeded => "budget_exceeded",
            Self::Skipped => "skipped",
            Self::Error => "error",
        };
        write!(f, "{}", s)
    }
}

/// What fallback was used when specialist failed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FallbackUsed {
    /// No fallback needed - specialist succeeded or was skipped
    None,
    /// Deterministic answerer produced the response
    Deterministic {
        /// The query class that was used for deterministic routing
        route_class: String,
    },
    /// Timeout fallback (v0.0.26)
    Timeout {
        /// The query class attempted before timeout
        route_class: String,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
}

impl std::fmt::Display for FallbackUsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Deterministic { route_class } => {
                write!(f, "deterministic ({})", route_class)
            }
            Self::Timeout {
                route_class,
                timeout_ms,
            } => {
                write!(f, "timeout ({}ms, {})", timeout_ms, route_class)
            }
        }
    }
}

/// Outcome of review stage (v0.0.26)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerOutcome {
    /// Deterministic gate accepted (no LLM needed)
    DeterministicAccept,
    /// Deterministic gate rejected (revise/escalate)
    DeterministicReject,
    /// Junior LLM review completed and accepted
    JuniorOk,
    /// Junior escalated to senior
    JuniorEscalated,
    /// Senior LLM review completed
    SeniorOk,
    /// Senior failed (final rejection)
    SeniorFailed,
    /// Reviewer timed out
    ReviewerTimeout,
    /// Reviewer budget exceeded
    ReviewerBudgetExceeded,
}

impl std::fmt::Display for ReviewerOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DeterministicAccept => "deterministic_accept",
            Self::DeterministicReject => "deterministic_reject",
            Self::JuniorOk => "junior_ok",
            Self::JuniorEscalated => "junior_escalated",
            Self::SeniorOk => "senior_ok",
            Self::SeniorFailed => "senior_failed",
            Self::ReviewerTimeout => "reviewer_timeout",
            Self::ReviewerBudgetExceeded => "reviewer_budget_exceeded",
        };
        write!(f, "{}", s)
    }
}
