//! Parse outcome types and conversions.

use crate::specialist_protocol::{
    fallback::FallbackReason,
    schema::StrictResponse,
    validation_types::ValidationResult,
};

/// Parse outcome with detailed error classification
#[derive(Debug, Clone)]
pub enum ParseOutcome {
    /// Successfully parsed and validated
    Success(StrictResponse, ValidationResult),
    /// Parsed but validation failed (response may be partially usable)
    ValidationFailed(StrictResponse, ValidationResult),
    /// No JSON found in output
    NoJson { raw: String },
    /// Invalid JSON syntax
    InvalidJson { raw: String, error: String },
    /// JSON parsed but doesn't match schema
    SchemaMismatch { raw: String, error: String },
    /// Timeout during parsing
    Timeout { elapsed_ms: u64 },
}

impl ParseOutcome {
    /// Check if parsing was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_, _))
    }

    /// Get response if available (even if validation failed)
    pub fn response(&self) -> Option<&StrictResponse> {
        match self {
            Self::Success(r, _) | Self::ValidationFailed(r, _) => Some(r),
            _ => None,
        }
    }

    /// Get validation result if available
    pub fn validation(&self) -> Option<&ValidationResult> {
        match self {
            Self::Success(_, v) | Self::ValidationFailed(_, v) => Some(v),
            _ => None,
        }
    }

    /// Convert to fallback reason if failed
    pub fn to_fallback_reason(&self) -> Option<FallbackReason> {
        match self {
            Self::Success(_, _) => None,
            Self::ValidationFailed(_, v) => {
                let errors: Vec<String> = v.errors.iter().map(|e| e.to_string()).collect();
                Some(FallbackReason::ValidationFailed(errors.join("; ")))
            }
            Self::NoJson { .. } => Some(FallbackReason::ParseError("No JSON found".to_string())),
            Self::InvalidJson { error, .. } => Some(FallbackReason::ParseError(error.clone())),
            Self::SchemaMismatch { error, .. } => Some(FallbackReason::ParseError(error.clone())),
            Self::Timeout { .. } => Some(FallbackReason::Timeout),
        }
    }
}

/// Truncate string for error messages
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Create a timeout parse outcome
pub fn timeout_outcome(elapsed_ms: u64) -> ParseOutcome {
    ParseOutcome::Timeout { elapsed_ms }
}
