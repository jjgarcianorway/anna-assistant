//! Decoder result and error types.

use super::super::envelope::ModelResultEnvelope;

/// Result of decoding model output.
#[derive(Debug, Clone)]
pub enum DecodeResult {
    /// Successfully decoded envelope.
    Success(ModelResultEnvelope),
    /// Decoding failed with reason.
    Failed(DecodeError),
}

impl DecodeResult {
    /// Check if successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Get envelope if successful.
    pub fn envelope(&self) -> Option<&ModelResultEnvelope> {
        match self {
            Self::Success(e) => Some(e),
            Self::Failed(_) => None,
        }
    }

    /// Get error if failed.
    pub fn error(&self) -> Option<&DecodeError> {
        match self {
            Self::Success(_) => None,
            Self::Failed(e) => Some(e),
        }
    }
}

/// Decode error types.
#[derive(Debug, Clone)]
pub enum DecodeError {
    /// Model call timed out (NOT a parse error).
    ModelTimeout {
        timeout_ms: u64,
        partial_output: Option<String>,
    },
    /// No frame markers found.
    NoFrame { raw_output: String },
    /// Frame was incomplete (missing end marker).
    IncompleteFrame { partial_content: String },
    /// Multiple frames found (protocol violation).
    MultipleFrames,
    /// JSON parsing failed after all repair attempts.
    JsonParseError {
        message: String,
        attempted_repairs: usize,
        raw_json: String,
    },
    /// Envelope validation failed.
    EnvelopeInvalid { issues: Vec<String> },
    /// Empty output from model.
    EmptyOutput,
}

impl DecodeError {
    /// Human-readable error message.
    pub fn message(&self) -> String {
        match self {
            Self::ModelTimeout { timeout_ms, .. } => {
                format!("Model call timed out after {}ms", timeout_ms)
            }
            Self::NoFrame { .. } => "No protocol frame markers found in output".to_string(),
            Self::IncompleteFrame { .. } => {
                "Protocol frame incomplete (missing end marker)".to_string()
            }
            Self::MultipleFrames => {
                "Multiple protocol frames found (protocol violation)".to_string()
            }
            Self::JsonParseError {
                message,
                attempted_repairs,
                ..
            } => {
                format!(
                    "JSON parse failed after {} repair attempts: {}",
                    attempted_repairs, message
                )
            }
            Self::EnvelopeInvalid { issues } => {
                format!("Envelope validation failed: {}", issues.join("; "))
            }
            Self::EmptyOutput => "Empty output from model".to_string(),
        }
    }

    /// Get any partial output available.
    pub fn partial_output(&self) -> Option<&str> {
        match self {
            Self::ModelTimeout { partial_output, .. } => partial_output.as_deref(),
            Self::NoFrame { raw_output } => Some(raw_output),
            Self::IncompleteFrame { partial_content } => Some(partial_content),
            Self::JsonParseError { raw_json, .. } => Some(raw_json),
            _ => None,
        }
    }

    /// Check if this is a timeout error (not a parse error).
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::ModelTimeout { .. })
    }

    /// Check if this is a parse/protocol error.
    pub fn is_parse_error(&self) -> bool {
        matches!(
            self,
            Self::NoFrame { .. }
                | Self::IncompleteFrame { .. }
                | Self::MultipleFrames
                | Self::JsonParseError { .. }
                | Self::EnvelopeInvalid { .. }
        )
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for DecodeError {}
