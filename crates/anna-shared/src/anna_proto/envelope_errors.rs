//! Error types - ModelError, ErrorCode.

use serde::{Deserialize, Serialize};

/// An error from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelError {
    /// Error code.
    pub code: ErrorCode,
    /// Human-readable message.
    pub message: String,
}

impl ModelError {
    /// Create a new error.
    pub fn new(code: ErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    /// Create insufficient evidence error.
    pub fn insufficient_evidence(message: &str) -> Self {
        Self::new(ErrorCode::InsufficientEvidence, message)
    }

    /// Create need clarification error.
    pub fn need_clarification(message: &str) -> Self {
        Self::new(ErrorCode::NeedClarification, message)
    }
}

/// Error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Not enough evidence to answer.
    InsufficientEvidence,
    /// Need clarification from user.
    NeedClarification,
    /// Model cannot handle this type of request.
    UnsupportedRequest,
    /// Internal model error.
    InternalError,
    /// Context too long.
    ContextTooLong,
    /// Unknown error.
    Unknown,
}
