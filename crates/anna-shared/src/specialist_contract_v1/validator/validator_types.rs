//! Validation types and constants - v0.0.440.

use super::super::schema::SpecialistResponseV1;

/// Maximum response length (rough estimate: 1 token ~= 4 chars).
pub const MAX_RESPONSE_TOKENS: usize = 500;
pub const MAX_RESPONSE_CHARS: usize = MAX_RESPONSE_TOKENS * 4;

/// Validation error types.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Empty response.
    Empty,
    /// Not valid JSON.
    InvalidJson { message: String },
    /// JSON parsed but schema invalid.
    SchemaInvalid { issues: Vec<String> },
    /// Case ID mismatch.
    CaseIdMismatch { expected: String, got: String },
    /// Contains markdown (forbidden).
    ContainsMarkdown { offending: String },
    /// Response too long (token estimate).
    TooLong { estimated_tokens: usize },
}

impl ValidationError {
    /// Get error code for logging.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "EMPTY",
            Self::InvalidJson { .. } => "INVALID_JSON",
            Self::SchemaInvalid { .. } => "SCHEMA_INVALID",
            Self::CaseIdMismatch { .. } => "CASE_ID_MISMATCH",
            Self::ContainsMarkdown { .. } => "CONTAINS_MARKDOWN",
            Self::TooLong { .. } => "TOO_LONG",
        }
    }

    /// Get human-readable message.
    pub fn message(&self) -> String {
        match self {
            Self::Empty => "Empty response".to_string(),
            Self::InvalidJson { message } => format!("Invalid JSON: {}", message),
            Self::SchemaInvalid { issues } => format!("Schema invalid: {}", issues.join(", ")),
            Self::CaseIdMismatch { expected, got } => {
                format!("Case ID mismatch: expected '{}', got '{}'", expected, got)
            }
            Self::ContainsMarkdown { offending } => {
                format!("Contains forbidden markdown: '{}'", offending)
            }
            Self::TooLong { estimated_tokens } => {
                format!("Response too long: ~{} tokens", estimated_tokens)
            }
        }
    }

    /// Whether this error might be fixable by retry.
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::SchemaInvalid { .. } | Self::ContainsMarkdown { .. } | Self::TooLong { .. }
        )
    }
}

/// Result of validation.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Valid response.
    Valid(SpecialistResponseV1),
    /// Invalid response.
    Invalid {
        error: ValidationError,
        raw_response: String,
    },
}

impl ValidationResult {
    /// Check if valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// Get the response if valid.
    pub fn response(&self) -> Option<&SpecialistResponseV1> {
        match self {
            Self::Valid(r) => Some(r),
            Self::Invalid { .. } => None,
        }
    }

    /// Get the error if invalid.
    pub fn error(&self) -> Option<&ValidationError> {
        match self {
            Self::Valid(_) => None,
            Self::Invalid { error, .. } => Some(error),
        }
    }
}
