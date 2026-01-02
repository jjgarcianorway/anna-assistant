//! Internal error types for ticket lifecycle.

use serde::{Deserialize, Serialize};

/// Internal error classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InternalError {
    /// JSON parsing failed after retries
    ParseError { attempts: u8, last_error: String },
    /// LLM timeout
    Timeout { timeout_ms: u64 },
    /// Probe execution failed
    ProbeFailure { probe_id: String, error: String },
    /// Unexpected crash/panic
    InternalCrash { context: String },
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError { attempts, .. } => write!(f, "parse_error (attempts: {})", attempts),
            Self::Timeout { timeout_ms } => write!(f, "timeout ({}ms)", timeout_ms),
            Self::ProbeFailure { probe_id, .. } => write!(f, "probe_failure ({})", probe_id),
            Self::InternalCrash { .. } => write!(f, "internal_crash"),
        }
    }
}
