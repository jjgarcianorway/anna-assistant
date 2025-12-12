//! Robust JSON parser for specialist responses (v0.0.425).
//!
//! Features:
//! - First parse attempt with standard JSON
//! - Retry with repair request on failure
//! - Safe error response synthesis on final failure
//! - No "Failed to parse" errors exposed to users

use super::{ErrorInfo, ErrorKind, ResponseStatus, SpecialistResponse, MAX_PARSE_RETRIES};
use serde_json::Value;

/// Parser result
#[derive(Debug)]
pub enum ParseResult {
    /// Successfully parsed response
    Success(SpecialistResponse),
    /// Needs repair - return the error message for LLM
    NeedsRepair(String),
    /// Final failure - use synthesized error response
    Failed(SpecialistResponse),
}

/// Parse a specialist response from raw text.
///
/// Strategy:
/// 1. Try to extract JSON from the text (handle markdown blocks)
/// 2. Parse the JSON into SpecialistResponse
/// 3. Validate the response structure
/// 4. Return appropriate result
pub fn parse_specialist_response(text: &str, ticket_id: &str) -> ParseResult {
    // Step 1: Extract JSON from text
    let json_str = extract_json(text);

    // Step 2: Try to parse
    match serde_json::from_str::<SpecialistResponse>(&json_str) {
        Ok(mut response) => {
            // Ensure ticket_id is set
            if response.ticket_id.is_empty() {
                response.ticket_id = ticket_id.to_string();
            }

            // Validate
            if let Err(errors) = response.validate() {
                return ParseResult::NeedsRepair(format!(
                    "JSON parsed but validation failed: {}",
                    errors.join(", ")
                ));
            }

            ParseResult::Success(response)
        }
        Err(e) => {
            // Generate repair instructions
            let repair_msg = generate_repair_message(&e, &json_str);
            ParseResult::NeedsRepair(repair_msg)
        }
    }
}

/// Parse with retry context (after repair attempt).
pub fn parse_with_retry(text: &str, ticket_id: &str, attempt: usize) -> ParseResult {
    if attempt > MAX_PARSE_RETRIES {
        // Final failure - synthesize error response
        return ParseResult::Failed(synthesize_parse_error(ticket_id, text));
    }

    parse_specialist_response(text, ticket_id)
}

/// Extract JSON from text, handling markdown code blocks.
fn extract_json(text: &str) -> String {
    let text = text.trim();

    // Try to find JSON in markdown code block
    if let Some(start) = text.find("```json") {
        if let Some(end) = text[start..]
            .find("```\n")
            .or_else(|| text[start..].rfind("```"))
        {
            let json_start = start + 7; // Skip "```json"
            if json_start < start + end {
                return text[json_start..start + end].trim().to_string();
            }
        }
    }

    // Try generic code block
    if let Some(start) = text.find("```") {
        let after_backticks = start + 3;
        // Skip language identifier if present
        let content_start = text[after_backticks..]
            .find('\n')
            .map(|i| after_backticks + i + 1)
            .unwrap_or(after_backticks);

        if let Some(end) = text[content_start..].find("```") {
            return text[content_start..content_start + end].trim().to_string();
        }
    }

    // Try to find raw JSON object
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }

    // Return as-is, let parser handle the error
    text.to_string()
}

/// Generate a repair message for the LLM.
fn generate_repair_message(error: &serde_json::Error, attempted_json: &str) -> String {
    let error_line = error.line();
    let error_col = error.column();

    // Get context around the error
    let lines: Vec<&str> = attempted_json.lines().collect();
    let context = if error_line > 0 && error_line <= lines.len() {
        let line_idx = error_line - 1;
        let mut ctx = String::new();
        if line_idx > 0 {
            ctx.push_str(&format!(
                "Line {}: {}\n",
                error_line - 1,
                lines[line_idx - 1]
            ));
        }
        ctx.push_str(&format!(
            "Line {} (error): {}\n",
            error_line, lines[line_idx]
        ));
        ctx.push_str(&format!("{}^\n", " ".repeat(error_col + 15)));
        if line_idx + 1 < lines.len() {
            ctx.push_str(&format!(
                "Line {}: {}\n",
                error_line + 1,
                lines[line_idx + 1]
            ));
        }
        ctx
    } else {
        String::new()
    };

    format!(
        "JSON parse error at line {}, column {}: {}\n\n{}\n\
        Please output ONLY valid JSON matching the SpecialistResponse schema. \
        Required fields: ticket_id, status, summary, confidence.",
        error_line, error_col, error, context
    )
}

/// Synthesize a safe error response when parsing completely fails.
fn synthesize_parse_error(ticket_id: &str, _raw_text: &str) -> SpecialistResponse {
    SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status: ResponseStatus::Error,
        summary: "Unable to process response".to_string(),
        confidence: 0.0,
        error: ErrorInfo {
            message: Some("The specialist encountered a formatting issue".to_string()),
            kind: Some(ErrorKind::ParseError),
            details: None, // Don't expose raw text to user
        },
        ..Default::default()
    }
}

/// Try to salvage partial data from malformed JSON.
pub fn try_salvage_response(text: &str, ticket_id: &str) -> Option<SpecialistResponse> {
    let json_str = extract_json(text);

    // Try to parse as generic Value first
    let value: Value = serde_json::from_str(&json_str).ok()?;
    let obj = value.as_object()?;

    // Extract what we can
    let summary = obj
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("Response available")
        .to_string();

    let status = obj
        .get("status")
        .and_then(|v| v.as_str())
        .and_then(parse_status)
        .unwrap_or(ResponseStatus::Partial);

    let confidence = obj
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.5);

    // Extract findings if present
    let findings = obj
        .get("findings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let key = f.get("key")?.as_str()?.to_string();
                    let value = f.get("value")?.as_str()?.to_string();
                    Some(super::Finding {
                        key,
                        value,
                        evidence_refs: vec![],
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract analysis bullets if present
    let analysis = obj
        .get("analysis")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(SpecialistResponse {
        ticket_id: ticket_id.to_string(),
        status,
        summary,
        confidence,
        findings,
        analysis,
        ..Default::default()
    })
}

/// Parse status string to enum.
fn parse_status(s: &str) -> Option<ResponseStatus> {
    match s {
        "success" => Some(ResponseStatus::Success),
        "partial" => Some(ResponseStatus::Partial),
        "no_data" => Some(ResponseStatus::NoData),
        "unsupported" => Some(ResponseStatus::Unsupported),
        "error" => Some(ResponseStatus::Error),
        _ => None,
    }
}

/// Repair request to send back to LLM.
#[derive(Debug, Clone)]
pub struct RepairRequest {
    /// Original ticket ID
    pub ticket_id: String,
    /// Error description
    pub error: String,
    /// Attempt number
    pub attempt: usize,
}

impl RepairRequest {
    /// Create a new repair request.
    pub fn new(ticket_id: &str, error: &str, attempt: usize) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            error: error.to_string(),
            attempt,
        }
    }

    /// Generate the repair prompt.
    pub fn to_prompt(&self) -> String {
        format!(
            "Your previous response had a JSON formatting error:\n{}\n\n\
            Please respond with ONLY a valid JSON object. No markdown, no explanation.\n\
            Required structure:\n\
            {{\n  \
              \"ticket_id\": \"{}\",\n  \
              \"status\": \"success|partial|no_data|unsupported|error\",\n  \
              \"summary\": \"one-line technical summary\",\n  \
              \"confidence\": 0.0-1.0,\n  \
              \"findings\": [...],\n  \
              \"analysis\": [...]\n\
            }}",
            self.error, self.ticket_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_markdown() {
        let text = r#"Here is the response:
```json
{"ticket_id": "DSK-001", "status": "success", "summary": "Test", "confidence": 0.9}
```
Done."#;

        let json = extract_json(text);
        assert!(json.contains("ticket_id"));
        assert!(json.contains("DSK-001"));
    }

    #[test]
    fn test_extract_json_raw() {
        let text =
            r#"{"ticket_id": "DSK-002", "status": "success", "summary": "Raw", "confidence": 0.8}"#;
        let json = extract_json(text);
        assert_eq!(json, text);
    }

    #[test]
    fn test_parse_valid_response() {
        let json = r#"{"ticket_id": "DSK-003", "status": "success", "summary": "Memory is healthy", "confidence": 0.95}"#;

        match parse_specialist_response(json, "DSK-003") {
            ParseResult::Success(resp) => {
                assert_eq!(resp.ticket_id, "DSK-003");
                assert_eq!(resp.status, ResponseStatus::Success);
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"ticket_id": "DSK-004", "status": "success" "summary": "Missing comma"}"#;

        match parse_specialist_response(json, "DSK-004") {
            ParseResult::NeedsRepair(msg) => {
                assert!(msg.contains("JSON parse error"));
            }
            other => panic!("Expected NeedsRepair, got {:?}", other),
        }
    }

    #[test]
    fn test_salvage_partial_response() {
        let json = r#"{"summary": "Partial data", "confidence": 0.6, "unknown_field": true}"#;

        let salvaged = try_salvage_response(json, "DSK-005");
        assert!(salvaged.is_some());
        let resp = salvaged.unwrap();
        assert_eq!(resp.summary, "Partial data");
        assert_eq!(resp.confidence, 0.6);
    }

    #[test]
    fn test_repair_request() {
        let req = RepairRequest::new("DSK-006", "Missing comma", 1);
        let prompt = req.to_prompt();

        assert!(prompt.contains("DSK-006"));
        assert!(prompt.contains("Missing comma"));
        assert!(prompt.contains("status"));
    }
}
