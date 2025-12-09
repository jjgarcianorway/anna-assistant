//! Ticket-related type definitions (v0.0.215).

use serde::{Deserialize, Serialize};

/// Default maximum junior verification rounds
pub const DEFAULT_JUNIOR_ROUNDS_MAX: u8 = 3;

/// Default maximum senior escalation rounds
pub const DEFAULT_SENIOR_ROUNDS_MAX: u8 = 1;

/// Default reliability threshold for verification
pub const DEFAULT_RELIABILITY_THRESHOLD: u8 = 80;

/// Default maximum clarification rounds
pub fn default_clarification_max() -> u8 {
    3 // Maximum 3 clarification rounds before giving up
}

/// Risk level for ticket actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Read-only operations (probes, queries)
    #[default]
    ReadOnly,
    /// Low-risk changes (config tweaks, service restarts)
    LowRiskChange,
    /// High-risk changes (package installs, disk operations)
    HighRiskChange,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "read-only"),
            Self::LowRiskChange => write!(f, "low-risk-change"),
            Self::HighRiskChange => write!(f, "high-risk-change"),
        }
    }
}

/// Ticket status in the service desk workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    /// Ticket created, not yet processed
    #[default]
    New,
    /// Awaiting user clarification (v0.0.31)
    AwaitingClarification,
    /// Verifying user's clarification answer (v0.0.31)
    VerifyingClarification,
    /// Running probes to gather evidence
    Probing,
    /// Answer drafted, awaiting verification
    AnswerDrafted,
    /// Verified by junior, meets reliability threshold
    Verified,
    /// Escalated to senior for review
    Escalated,
    /// Failed to meet reliability after all attempts
    Failed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::AwaitingClarification => write!(f, "awaiting-clarification"),
            Self::VerifyingClarification => write!(f, "verifying-clarification"),
            Self::Probing => write!(f, "probing"),
            Self::AnswerDrafted => write!(f, "answer-drafted"),
            Self::Verified => write!(f, "verified"),
            Self::Escalated => write!(f, "escalated"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
