//! JSON parser for specialist responses (v0.0.428).
//!
//! Parses specialist JSON with multiple fallback strategies.
//! Never returns raw error messages to users.

use super::{
    fallback::{FallbackContext, FallbackReason},
    schema::{ResponseMeta, ResponseStatus, StrictResponse},
    validator::{validate_response, ValidationResult},
};
use serde_json;

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

/// Parse a specialist response from raw output
pub fn parse_specialist_response(raw: &str) -> ParseOutcome {
    // Step 1: Extract JSON from raw output
    let json_str = match extract_json(raw) {
        Some(j) => j,
        None => return ParseOutcome::NoJson { raw: truncate(raw, 500) },
    };

    // Step 2: Parse JSON
    let response: StrictResponse = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(e) => {
            // Try lenient parsing with defaults
            match try_lenient_parse(&json_str) {
                Some(r) => r,
                None => {
                    return ParseOutcome::InvalidJson {
                        raw: truncate(raw, 500),
                        error: e.to_string(),
                    }
                }
            }
        }
    };

    // Step 3: Validate response
    let validation = validate_response(&response);

    if validation.valid {
        ParseOutcome::Success(response, validation)
    } else {
        // Check if errors are severe enough to reject
        let has_severe_errors = validation.errors.iter().any(|e| {
            matches!(e,
                super::ValidationError::InventedData(_)
                | super::ValidationError::ForbiddenPattern(_)
                | super::ValidationError::EmptySummary
            )
        });

        if has_severe_errors {
            ParseOutcome::ValidationFailed(response, validation)
        } else {
            // Warnings only - still successful
            ParseOutcome::Success(response, validation)
        }
    }
}

/// Extract JSON object from raw text using multiple strategies
fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Strategy 1: Clean JSON object at start
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Strategy 2: Markdown JSON code block
    if let Some(start) = trimmed.find("```json") {
        let after_marker = &trimmed[start + 7..];
        if let Some(end) = after_marker.find("```") {
            let json_content = after_marker[..end].trim();
            if json_content.starts_with('{') && json_content.ends_with('}') {
                return Some(json_content.to_string());
            }
        }
    }

    // Strategy 3: Generic code block
    if let Some(start) = trimmed.find("```") {
        let after_marker = &trimmed[start + 3..];
        // Skip language identifier if present
        let content_start = after_marker.find('\n').map(|i| i + 1).unwrap_or(0);
        let after_newline = &after_marker[content_start..];
        if let Some(end) = after_newline.find("```") {
            let json_content = after_newline[..end].trim();
            if json_content.starts_with('{') && json_content.ends_with('}') {
                return Some(json_content.to_string());
            }
        }
    }

    // Strategy 4: Find first { to last }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return Some(trimmed[start..=end].to_string());
            }
        }
    }

    None
}

/// Try lenient parsing with sensible defaults
fn try_lenient_parse(json_str: &str) -> Option<StrictResponse> {
    // Try parsing as a partial object and fill in defaults
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let obj = value.as_object()?;

    // Extract fields with defaults
    let status = obj.get("status")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "success" => Some(ResponseStatus::Success),
            "partial" => Some(ResponseStatus::Partial),
            "failure" => Some(ResponseStatus::Failure),
            _ => None,
        })
        .unwrap_or(ResponseStatus::Failure);

    let confidence = obj.get("confidence")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.0);

    let domain = obj.get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let intent = obj.get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let summary = obj.get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Try to get key_facts from details
    let key_facts: Vec<String> = obj.get("details")
        .and_then(|d| d.get("key_facts"))
        .and_then(|kf| kf.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let meta = ResponseMeta {
        handled_by: obj.get("meta")
            .and_then(|m| m.get("handled_by"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        ticket_id: obj.get("meta")
            .and_then(|m| m.get("ticket_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        version: 1,
    };

    Some(StrictResponse {
        status,
        confidence,
        domain,
        intent,
        summary,
        details: super::ResponseDetails {
            key_facts,
            diagnosis: None,
            recommendations: vec![],
        },
        actions: Default::default(),
        evidence: Default::default(),
        metrics: Default::default(),
        meta,
    })
}

/// Truncate string for error messages
fn truncate(s: &str, max: usize) -> String {
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

/// Parse with timeout handling
pub fn parse_with_timeout(
    raw: &str,
    timeout_ms: u64,
    elapsed_ms: u64,
) -> ParseOutcome {
    if elapsed_ms >= timeout_ms {
        return timeout_outcome(elapsed_ms);
    }

    parse_specialist_response(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clean_json() {
        let json = r#"{
            "status": "success",
            "confidence": 0.9,
            "domain": "services.systemd",
            "intent": "check_failed_services",
            "summary": "No failed systemd services.",
            "details": {
                "key_facts": ["0 failed units"],
                "diagnosis": null,
                "recommendations": []
            },
            "actions": { "proposed": [], "auto_applied": [] },
            "evidence": { "probes_used": [], "arch_wiki_pages": [], "man_pages": [], "help_commands": [] },
            "metrics": { "latency_ms": 100, "tokens_in": 500, "tokens_out": 200 },
            "meta": { "handled_by": "Sofia", "ticket_id": "DSK-001", "version": 1 }
        }"#;

        let result = parse_specialist_response(json);
        assert!(result.is_success());

        let response = result.response().unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(response.summary, "No failed systemd services.");
    }

    #[test]
    fn test_parse_markdown_json() {
        let raw = r#"
Here is my analysis:

```json
{
    "status": "success",
    "confidence": 0.85,
    "domain": "system",
    "intent": "check_memory",
    "summary": "16GB RAM available.",
    "details": { "key_facts": [] },
    "evidence": {},
    "meta": { "handled_by": "Test", "ticket_id": "T-1" }
}
```

That's all.
"#;

        let result = parse_specialist_response(raw);
        assert!(result.is_success());
    }

    #[test]
    fn test_parse_no_json() {
        let raw = "This is just plain text with no JSON.";
        let result = parse_specialist_response(raw);
        assert!(matches!(result, ParseOutcome::NoJson { .. }));
    }

    #[test]
    fn test_parse_invalid_json() {
        let raw = "{ invalid json syntax }";
        let result = parse_specialist_response(raw);
        // May parse leniently or fail
        assert!(matches!(result, ParseOutcome::InvalidJson { .. } | ParseOutcome::ValidationFailed(_, _)));
    }

    #[test]
    fn test_lenient_parsing() {
        let json = r#"{
            "status": "success",
            "summary": "It works",
            "meta": { "ticket_id": "T-1" }
        }"#;

        let result = parse_specialist_response(json);
        // Should parse with defaults
        let response = result.response();
        assert!(response.is_some());
    }

    #[test]
    fn test_timeout_outcome() {
        let result = timeout_outcome(5000);
        assert!(matches!(result, ParseOutcome::Timeout { elapsed_ms: 5000 }));

        let reason = result.to_fallback_reason();
        assert!(matches!(reason, Some(FallbackReason::Timeout)));
    }

    #[test]
    fn test_extract_json_strategies() {
        // Clean JSON
        assert!(extract_json(r#"{"test": 1}"#).is_some());

        // With whitespace
        assert!(extract_json(r#"   {"test": 1}   "#).is_some());

        // In markdown block
        assert!(extract_json("```json\n{\"test\": 1}\n```").is_some());

        // Embedded in text
        assert!(extract_json("Here is the result: {\"test\": 1} end").is_some());

        // No JSON
        assert!(extract_json("No JSON here").is_none());
    }

    #[test]
    fn test_validation_failure_detection() {
        let json = r#"{
            "status": "success",
            "confidence": 0.9,
            "domain": "system",
            "intent": "check_installed",
            "summary": "unknown is installed",
            "details": {},
            "evidence": {},
            "meta": { "handled_by": "Test", "ticket_id": "T-1" }
        }"#;

        let result = parse_specialist_response(json);
        assert!(matches!(result, ParseOutcome::ValidationFailed(_, _)));
    }
}
