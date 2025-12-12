//! Two-Stage Decoder (Part B) - v0.0.436.
//!
//! Decoder pipeline that never blocks:
//! 1. Fast path: Extract framed JSON
//! 2. Recovery path: Tolerant JSON scanning and repair
//!
//! Parsing itself NEVER times out - only model calls can timeout.

use super::envelope::ModelResultEnvelope;
use super::framing::{extract_framed_content, FrameResult};
use serde_json;

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
    NoFrame {
        raw_output: String,
    },
    /// Frame was incomplete (missing end marker).
    IncompleteFrame {
        partial_content: String,
    },
    /// Multiple frames found (protocol violation).
    MultipleFrames,
    /// JSON parsing failed after all repair attempts.
    JsonParseError {
        message: String,
        attempted_repairs: usize,
        raw_json: String,
    },
    /// Envelope validation failed.
    EnvelopeInvalid {
        issues: Vec<String>,
    },
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
            Self::MultipleFrames => "Multiple protocol frames found (protocol violation)".to_string(),
            Self::JsonParseError { message, attempted_repairs, .. } => {
                format!("JSON parse failed after {} repair attempts: {}", attempted_repairs, message)
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

/// Protocol decoder with repair capabilities.
pub struct ProtoDecoder {
    /// Maximum JSON repair attempts.
    max_repairs: usize,
}

impl ProtoDecoder {
    /// Create a new decoder.
    pub fn new() -> Self {
        Self {
            max_repairs: super::MAX_REPAIR_ATTEMPTS,
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
    fn parse_json(&self, content: &str) -> DecodeResult {
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
                json_str = remove_trailing_commas(&json_str);
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
            super::envelope::EnvelopeValidation::Valid => DecodeResult::Success(envelope),
            super::envelope::EnvelopeValidation::Invalid { issues } => {
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

/// Remove trailing commas from JSON (common model error).
fn remove_trailing_commas(json: &str) -> String {
    // Simple regex-free approach: remove comma before } or ]
    let mut result = json.to_string();

    // Remove ", }" patterns
    while result.contains(", }") {
        result = result.replace(", }", " }");
    }
    while result.contains(",}") {
        result = result.replace(",}", "}");
    }

    // Remove ", ]" patterns
    while result.contains(", ]") {
        result = result.replace(", ]", " ]");
    }
    while result.contains(",]") {
        result = result.replace(",]", "]");
    }

    // Remove trailing comma before closing (with newlines)
    let lines: Vec<&str> = result.lines().collect();
    if lines.len() > 1 {
        let mut new_lines = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if i < lines.len() - 1 {
                let next_trimmed = lines.get(i + 1).map(|l| l.trim()).unwrap_or("");
                if trimmed.ends_with(',') && (next_trimmed.starts_with('}') || next_trimmed.starts_with(']')) {
                    new_lines.push(line.trim_end_matches(','));
                    continue;
                }
            }
            new_lines.push(line);
        }
        result = new_lines.join("\n");
    }

    result
}

/// Extract the largest JSON object from text.
fn extract_json_object(text: &str) -> Option<String> {
    // Find first '{' and last matching '}'
    let first_brace = text.find('{')?;

    // Count braces to find matching close
    let mut depth = 0;
    let mut last_close = None;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in text[first_brace..].char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    last_close = Some(first_brace + i);
                    break;
                }
            }
            _ => {}
        }
    }

    if let Some(end) = last_close {
        Some(text[first_brace..=end].to_string())
    } else {
        None
    }
}

/// Truncate output for error messages.
fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...[truncated]", &output[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::framing::{PROTO_START, PROTO_END};

    fn valid_envelope_json() -> String {
        r#"{
            "ok": true,
            "role": "junior",
            "ticket_id": "DSK-001",
            "confidence": 0.85,
            "summary": "Boot time is 15 seconds",
            "claims": [],
            "next_actions": [],
            "evidence_used": [],
            "errors": []
        }"#.to_string()
    }

    #[test]
    fn test_decode_valid_framed() {
        let decoder = ProtoDecoder::new();
        let output = format!("{}\n{}\n{}", PROTO_START, valid_envelope_json(), PROTO_END);

        let result = decoder.decode(&output);
        assert!(result.is_success());

        let envelope = result.envelope().unwrap();
        assert!(envelope.ok);
        assert_eq!(envelope.ticket_id, "DSK-001");
    }

    #[test]
    fn test_decode_with_preamble() {
        let decoder = ProtoDecoder::new();
        let output = format!(
            "Some thinking text...\n{}\n{}\n{}\nMore text",
            PROTO_START, valid_envelope_json(), PROTO_END
        );

        let result = decoder.decode(&output);
        assert!(result.is_success());
    }

    #[test]
    fn test_decode_no_frame_with_recovery() {
        let decoder = ProtoDecoder::new();
        // Raw JSON without framing
        let output = valid_envelope_json();

        let result = decoder.decode(&output);
        assert!(result.is_success(), "Should recover JSON without frame");
    }

    #[test]
    fn test_decode_trailing_comma() {
        let decoder = ProtoDecoder::new();
        let json_with_comma = r#"{
            "ok": true,
            "role": "junior",
            "ticket_id": "DSK-001",
            "confidence": 0.85,
            "summary": "Test",
            "claims": [],
            "next_actions": [],
            "evidence_used": [],
            "errors": [],
        }"#;

        let output = format!("{}\n{}\n{}", PROTO_START, json_with_comma, PROTO_END);
        let result = decoder.decode(&output);
        assert!(result.is_success(), "Should repair trailing comma");
    }

    #[test]
    fn test_decode_empty_output() {
        let decoder = ProtoDecoder::new();
        let result = decoder.decode("");

        assert!(!result.is_success());
        assert!(matches!(result.error(), Some(DecodeError::EmptyOutput)));
    }

    #[test]
    fn test_decode_incomplete_frame() {
        let decoder = ProtoDecoder::new();
        let output = format!("{}\n{}", PROTO_START, valid_envelope_json());
        // No end marker

        let result = decoder.decode(&output);
        // Should still succeed because JSON is complete
        assert!(result.is_success(), "Should parse complete JSON even without end marker");
    }

    #[test]
    fn test_decode_multiple_frames() {
        let decoder = ProtoDecoder::new();
        let output = format!(
            "{}\n{{}}\n{}\n{}\n{{}}\n{}",
            PROTO_START, PROTO_END, PROTO_START, PROTO_END
        );

        let result = decoder.decode(&output);
        assert!(!result.is_success());
        assert!(matches!(result.error(), Some(DecodeError::MultipleFrames)));
    }

    #[test]
    fn test_timeout_error() {
        let result = ProtoDecoder::timeout_error(12000, Some("partial...".to_string()));

        assert!(!result.is_success());
        if let Some(DecodeError::ModelTimeout { timeout_ms, partial_output }) = result.error() {
            assert_eq!(*timeout_ms, 12000);
            assert!(partial_output.is_some());
            assert!(result.error().unwrap().is_timeout());
            assert!(!result.error().unwrap().is_parse_error());
        } else {
            panic!("Expected ModelTimeout error");
        }
    }

    #[test]
    fn test_extract_json_object() {
        let text = "Some text { \"key\": \"value\" } more text";
        let json = extract_json_object(text);
        assert!(json.is_some());
        assert!(json.unwrap().contains("key"));

        let nested = r#"Prefix {"outer": {"inner": 1}} suffix"#;
        let json = extract_json_object(nested);
        assert!(json.is_some());
        assert!(json.unwrap().contains("inner"));
    }

    #[test]
    fn test_remove_trailing_commas() {
        let with_comma = r#"{"a": 1, "b": 2,}"#;
        let fixed = remove_trailing_commas(with_comma);
        assert!(!fixed.ends_with(",}"));

        let array_comma = r#"[1, 2, 3,]"#;
        let fixed = remove_trailing_commas(array_comma);
        assert!(!fixed.ends_with(",]"));
    }

    #[test]
    fn test_decode_error_display() {
        let err = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };
        assert!(err.message().contains("12000"));

        let err = DecodeError::NoFrame {
            raw_output: "test".to_string(),
        };
        assert!(err.message().contains("frame"));
    }
}
