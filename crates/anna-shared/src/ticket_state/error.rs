//! Error classification for failed tickets

use serde::{Deserialize, Serialize};

use super::outcome::TicketOutcome;

/// Error classification for failed tickets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// LLM call exceeded timeout
    LlmTimeout,
    /// LLM output could not be parsed as JSON
    LlmParseError,
    /// Probe execution failed
    ProbeFailure,
    /// Internal system error
    InternalError,
    /// Query type not supported
    Unsupported,
    /// Validation failed after all retries
    ValidationFailed,
    /// User cancelled the request
    Cancelled,
    /// v0.0.411: Not enough evidence to answer safely
    MissingEvidence,
    /// v0.0.411: Too risky to answer (could cause damage)
    UnsafeToAnswer,
}

impl ErrorKind {
    /// Convert error kind to ticket outcome
    pub fn to_outcome(&self) -> TicketOutcome {
        match self {
            Self::LlmTimeout => TicketOutcome::ErrorTimeout,
            Self::LlmParseError => TicketOutcome::ErrorParse,
            Self::ValidationFailed => TicketOutcome::ErrorParse,
            Self::ProbeFailure => TicketOutcome::ErrorTool,
            Self::InternalError => TicketOutcome::ErrorInternal,
            Self::Unsupported => TicketOutcome::CannotAnswerSafely,
            Self::Cancelled => TicketOutcome::CannotAnswerSafely,
            Self::MissingEvidence => TicketOutcome::CannotAnswerSafely,
            Self::UnsafeToAnswer => TicketOutcome::CannotAnswerSafely,
        }
    }

    /// Check if this error is an LLM-related failure
    pub fn is_llm_error(&self) -> bool {
        matches!(
            self,
            Self::LlmTimeout | Self::LlmParseError | Self::ValidationFailed
        )
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LlmTimeout => write!(f, "llm_timeout"),
            Self::LlmParseError => write!(f, "llm_parse_error"),
            Self::ProbeFailure => write!(f, "probe_failure"),
            Self::InternalError => write!(f, "internal_error"),
            Self::Unsupported => write!(f, "unsupported"),
            Self::ValidationFailed => write!(f, "validation_failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::MissingEvidence => write!(f, "missing_evidence"),
            Self::UnsafeToAnswer => write!(f, "unsafe_to_answer"),
        }
    }
}
