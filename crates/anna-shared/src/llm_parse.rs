//! Strict LLM response parsing with proper error handling (v0.0.407).
//!
//! Implements a clean contract for parsing LLM JSON responses:
//! - Strict JSON validation
//! - Error classification
//! - Debug logging of raw output on failure
//! - No hand-waving of errors

use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use tracing::{error, warn};

/// Result of attempting to parse LLM output
#[derive(Debug)]
pub enum ParseResult<T> {
    /// Successfully parsed
    Ok(T),
    /// No JSON found in output
    NoJson { raw: String },
    /// JSON found but invalid structure
    InvalidJson { raw: String, error: String },
    /// JSON valid but doesn't match expected schema
    SchemaMismatch { raw: String, error: String },
}

impl<T> ParseResult<T> {
    /// Check if parsing succeeded
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Get the parsed value if successful
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Ok(v) => Some(v),
            _ => None,
        }
    }

    /// Get error description
    pub fn error_description(&self) -> Option<String> {
        match self {
            Self::Ok(_) => None,
            Self::NoJson { .. } => Some("No JSON object found in LLM output".to_string()),
            Self::InvalidJson { error, .. } => Some(format!("Invalid JSON: {}", error)),
            Self::SchemaMismatch { error, .. } => Some(format!("Schema mismatch: {}", error)),
        }
    }

    /// Get raw output for logging
    pub fn raw_output(&self) -> Option<&str> {
        match self {
            Self::Ok(_) => None,
            Self::NoJson { raw } => Some(raw),
            Self::InvalidJson { raw, .. } => Some(raw),
            Self::SchemaMismatch { raw, .. } => Some(raw),
        }
    }
}

/// Parse LLM output as JSON with strict validation
///
/// This function:
/// 1. Trims whitespace
/// 2. Extracts JSON object from output (handles markdown blocks)
/// 3. Validates JSON syntax
/// 4. Deserializes to the expected type
///
/// On failure, logs the raw output to a debug file.
pub fn parse_strict<T: DeserializeOwned>(raw: &str, ticket_id: &str) -> ParseResult<T> {
    let trimmed = raw.trim();

    // Step 1: Extract JSON object
    let json_str = match extract_json_object(trimmed) {
        Some(s) => s,
        None => {
            log_parse_failure(ticket_id, raw, "No JSON object found");
            return ParseResult::NoJson { raw: raw.to_string() };
        }
    };

    // Step 2: Validate JSON syntax
    let json_value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            let error = format!("JSON syntax error: {}", e);
            log_parse_failure(ticket_id, raw, &error);
            return ParseResult::InvalidJson {
                raw: raw.to_string(),
                error,
            };
        }
    };

    // Step 3: Deserialize to expected type
    match serde_json::from_value(json_value) {
        Ok(parsed) => ParseResult::Ok(parsed),
        Err(e) => {
            let error = format!("Schema mismatch: {}", e);
            log_parse_failure(ticket_id, raw, &error);
            ParseResult::SchemaMismatch {
                raw: raw.to_string(),
                error,
            }
        }
    }
}

/// Extract JSON object from LLM output
///
/// Handles common cases:
/// - Clean JSON starting with {
/// - JSON wrapped in markdown code blocks
/// - JSON with surrounding prose
fn extract_json_object(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Case 1: Clean JSON starting with {
    if trimmed.starts_with('{') {
        if let Some(end) = find_matching_brace(trimmed) {
            return Some(trimmed[..=end].to_string());
        }
    }

    // Case 2: JSON in markdown code block
    if let Some(start) = raw.find("```json") {
        let after_marker = &raw[start + 7..];
        if let Some(end) = after_marker.find("```") {
            let content = after_marker[..end].trim();
            if content.starts_with('{') {
                return Some(content.to_string());
            }
        }
    }

    // Case 3: JSON in generic code block
    if let Some(start) = raw.find("```") {
        let after_marker = &raw[start + 3..];
        // Skip language identifier if present
        let content_start = after_marker.find('\n').map(|i| i + 1).unwrap_or(0);
        let content = &after_marker[content_start..];
        if let Some(end) = content.find("```") {
            let json_content = content[..end].trim();
            if json_content.starts_with('{') {
                return Some(json_content.to_string());
            }
        }
    }

    // Case 4: Find { anywhere and extract to matching }
    if let Some(start) = raw.find('{') {
        let from_brace = &raw[start..];
        if let Some(end) = find_matching_brace(from_brace) {
            return Some(from_brace[..=end].to_string());
        }
    }

    None
}

/// Find the index of the closing brace that matches the opening brace
fn find_matching_brace(s: &str) -> Option<usize> {
    if !s.starts_with('{') {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in s.char_indices() {
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
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Log parse failure to debug file
fn log_parse_failure(ticket_id: &str, raw: &str, reason: &str) {
    warn!("LLM parse failure for {}: {}", ticket_id, reason);

    // Log to file for debugging
    let log_dir = parse_failure_log_dir();
    if fs::create_dir_all(&log_dir).is_err() {
        error!("Failed to create parse failure log directory");
        return;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filename = format!("{}_{}.txt", ticket_id, timestamp);
    let path = log_dir.join(filename);

    let content = format!(
        "Ticket: {}\nTimestamp: {}\nReason: {}\n\n--- Raw Output ---\n{}\n",
        ticket_id, timestamp, reason, raw
    );

    match fs::write(&path, content) {
        Ok(_) => warn!("Raw output logged to: {}", path.display()),
        Err(e) => error!("Failed to log parse failure: {}", e),
    }
}

/// Get directory for parse failure logs
fn parse_failure_log_dir() -> PathBuf {
    // Try /var/lib/anna/debug first
    let var_lib = PathBuf::from("/var/lib/anna/debug/parse_failures");
    if var_lib.parent().map(|p| p.exists()).unwrap_or(false) {
        return var_lib;
    }

    // Fall back to ~/.anna/debug
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
        .join("debug")
        .join("parse_failures")
}

/// Convenience function to parse and get Result
pub fn parse_llm_json<T: DeserializeOwned>(
    raw: &str,
    ticket_id: &str,
) -> Result<T, String> {
    match parse_strict(raw, ticket_id) {
        ParseResult::Ok(v) => Ok(v),
        ParseResult::NoJson { .. } => {
            Err("No JSON object found in LLM output".to_string())
        }
        ParseResult::InvalidJson { error, .. } => Err(error),
        ParseResult::SchemaMismatch { error, .. } => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestResponse {
        status: String,
        value: i32,
    }

    #[test]
    fn test_parse_clean_json() {
        let raw = r#"{"status": "ok", "value": 42}"#;
        let result: ParseResult<TestResponse> = parse_strict(raw, "TEST-001");
        assert!(result.is_ok());
        let parsed = result.ok().unwrap();
        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.value, 42);
    }

    #[test]
    fn test_parse_json_with_prose() {
        let raw = r#"Here is the response:
{"status": "ok", "value": 42}
That's all."#;
        let result: ParseResult<TestResponse> = parse_strict(raw, "TEST-002");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_json_in_markdown() {
        let raw = r#"```json
{"status": "ok", "value": 42}
```"#;
        let result: ParseResult<TestResponse> = parse_strict(raw, "TEST-003");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_no_json() {
        let raw = "This is just text without any JSON.";
        let result: ParseResult<TestResponse> = parse_strict(raw, "TEST-004");
        assert!(matches!(result, ParseResult::NoJson { .. }));
        assert_eq!(
            result.error_description(),
            Some("No JSON object found in LLM output".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_json() {
        let raw = r#"{"status": "ok", "value": }"#; // Invalid JSON
        let result: ParseResult<TestResponse> = parse_strict(raw, "TEST-005");
        assert!(matches!(result, ParseResult::InvalidJson { .. }));
    }

    #[test]
    fn test_parse_schema_mismatch() {
        let raw = r#"{"different_field": "value"}"#;
        let result: ParseResult<TestResponse> = parse_strict(raw, "TEST-006");
        assert!(matches!(result, ParseResult::SchemaMismatch { .. }));
    }

    #[test]
    fn test_find_matching_brace() {
        assert_eq!(find_matching_brace(r#"{"a": 1}"#), Some(7));
        assert_eq!(find_matching_brace(r#"{"a": {"b": 2}}"#), Some(14));
        assert_eq!(find_matching_brace(r#"{"a": "}"}"#), Some(9)); // Handle } in string
        assert_eq!(find_matching_brace("not json"), None);
    }

    #[test]
    fn test_extract_json_object() {
        // Clean JSON
        assert!(extract_json_object(r#"{"a": 1}"#).is_some());

        // With prose
        let with_prose = r#"Response: {"a": 1} done"#;
        let extracted = extract_json_object(with_prose);
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap(), r#"{"a": 1}"#);

        // No JSON
        assert!(extract_json_object("just text").is_none());
    }
}
