//! Ticket lifecycle state enum

use serde::{Deserialize, Serialize};

/// Ticket lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketState {
    /// Initial creation
    Created,
    /// Translator done, probes selected
    Planned,
    /// Probe results available
    ProbesRun,
    /// Documentation attached (optional)
    DocsAttached,
    /// LLM request sent (for solver path)
    LlmRequested,
    /// LLM failed (parse error, timeout, or explicit failure)
    LlmFailed,
    /// Final answer produced
    Answered,
    /// Commands executed (if any changes)
    CommandsRun,
    /// Successfully completed
    Success,
    /// Terminal failure state
    Failed,
}

impl Default for TicketState {
    fn default() -> Self {
        Self::Created
    }
}

impl std::fmt::Display for TicketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Planned => write!(f, "planned"),
            Self::ProbesRun => write!(f, "probes_run"),
            Self::DocsAttached => write!(f, "docs_attached"),
            Self::LlmRequested => write!(f, "llm_requested"),
            Self::LlmFailed => write!(f, "llm_failed"),
            Self::Answered => write!(f, "answered"),
            Self::CommandsRun => write!(f, "commands_run"),
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
