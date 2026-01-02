//! Core decoder implementation.

use super::helpers::{extract_json_object, truncate_output};
use super::types::{DecodeError, DecodeResult};
use super::super::envelope::ModelResultEnvelope;
use super::super::framing::{extract_framed_content, FrameResult};
use serde_json;

/// Protocol decoder with repair capabilities.
pub struct ProtoDecoder {
    /// Maximum JSON repair attempts.
    max_repairs: usize,
}

impl ProtoDecoder {
    /// Create a new decoder.
    pub fn new() -> Self {
        Self {
            max_repairs: super::super::MAX_REPAIR_ATTEMPTS,
        }
    }

    /// Create decoder with custom repair limit.
    pub fn with_max_repairs(max_repairs: usize) -> Self {
        Self { max_repairs }
    }

    /// Decode model output to envelope.
    ///
    /// This NEVER times out - it processes whatever output is available.
    pub fn decode(&self, output: &str) -> DecodeResult {
        // Check for empty output
        if output.trim().is_empty() {
            return DecodeResult::Failed(DecodeError::EmptyOutput);
        }

        // Stage 1: Fast path - try framed content
        match extract_framed_content(output) {
            FrameResult::Found { content, .. } => {
                return self.parse_json(&content);
            }
            FrameResult::NoFrame => {
                // Fall through to recovery path
            }
            FrameResult::IncompleteFrame { partial_content } => {
                // Try to parse partial content (maybe JSON is complete)
                if let DecodeResult::Success(env) = self.parse_json(&partial_content) {
                    return DecodeResult::Success(env);
                }
                return DecodeResult::Failed(DecodeError::IncompleteFrame { partial_content });
            }
            FrameResult::MultipleFrames => {
                return DecodeResult::Failed(DecodeError::MultipleFrames);
            }
        }

        // Stage 2: Recovery path - try to extract JSON without framing
        self.recover_json(output)
    }

    /// Parse JSON content with repair attempts.
    pub(super) fn parse_json(&self, content: &str) -> DecodeResult {
        let trimmed = content.trim();

        // Try direct parse first
        if let Ok(envelope) = serde_json::from_str::<ModelResultEnvelope>(trimmed) {
            return self.validate_envelope(envelope);
        }

        // Try repairs
        let mut attempts = 0;
        let mut json_str = trimmed.to_string();

        while attempts < self.max_repairs {
            attempts += 1;

            // Repair attempt 1: Remove trailing commas
            if attempts == 1 {
                json_str = super::helpers::remove_trailing_commas(&json_str);
            }

            // Repair attempt 2: Strip non-JSON text
            if attempts == 2 {
                json_str = extract_json_object(&json_str).unwrap_or(json_str);
            }

            if let Ok(envelope) = serde_json::from_str::<ModelResultEnvelope>(&json_str) {
                return self.validate_envelope(envelope);
            }
        }

        // All repairs failed
        let err_msg = match serde_json::from_str::<ModelResultEnvelope>(trimmed) {
            Err(e) => e.to_string(),
            Ok(_) => "unknown".to_string(), // Shouldn't happen
        };

        DecodeResult::Failed(DecodeError::JsonParseError {
            message: err_msg,
            attempted_repairs: attempts,
            raw_json: trimmed.to_string(),
        })
    }

    /// Validate envelope integrity.
    fn validate_envelope(&self, envelope: ModelResultEnvelope) -> DecodeResult {
        match envelope.validate() {
            super::super::envelope::EnvelopeValidation::Valid => DecodeResult::Success(envelope),
            super::super::envelope::EnvelopeValidation::Invalid { issues } => {
                DecodeResult::Failed(DecodeError::EnvelopeInvalid { issues })
            }
        }
    }

    /// Recovery path: Try to extract JSON from unframed output.
    fn recover_json(&self, output: &str) -> DecodeResult {
        // Try to find JSON object in output
        if let Some(json_str) = extract_json_object(output) {
            return self.parse_json(&json_str);
        }

        DecodeResult::Failed(DecodeError::NoFrame {
            raw_output: truncate_output(output, 500),
        })
    }

    /// Create a timeout error result.
    pub fn timeout_error(timeout_ms: u64, partial_output: Option<String>) -> DecodeResult {
        DecodeResult::Failed(DecodeError::ModelTimeout {
            timeout_ms,
            partial_output,
        })
    }
}

impl Default for ProtoDecoder {
    fn default() -> Self {
        Self::new()
    }
}
