//! Core parsing logic for specialist responses.

use super::{
    extraction::extract_json,
    lenient::try_lenient_parse,
    types::{truncate, ParseOutcome},
};
use crate::specialist_protocol::{
    schema::{ResponseStatus, StrictResponse},
    validation_core::validate_response,
    ValidationError,
};
use serde_json;

/// Parse a specialist response from raw output
pub fn parse_specialist_response(raw: &str) -> ParseOutcome {
    // Step 1: Extract JSON from raw output
    let json_str = match extract_json(raw) {
        Some(j) => j,
        None => {
            return ParseOutcome::NoJson {
                raw: truncate(raw, 500),
            }
        }
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
            matches!(
                e,
                ValidationError::InventedData(_)
                    | ValidationError::ForbiddenPattern(_)
                    | ValidationError::EmptySummary
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

/// Parse with timeout handling
pub fn parse_with_timeout(raw: &str, timeout_ms: u64, elapsed_ms: u64) -> ParseOutcome {
    if elapsed_ms >= timeout_ms {
        return super::types::timeout_outcome(elapsed_ms);
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
        assert!(matches!(
            result,
            ParseOutcome::InvalidJson { .. } | ParseOutcome::ValidationFailed(_, _)
        ));
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
