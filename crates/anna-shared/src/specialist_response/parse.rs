//! Parsing and extraction utilities for specialist responses.

use super::types::{ParseOutcome, UnifiedSpecialistResponse};

/// Extract JSON from raw LLM output
pub fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Try clean JSON object first
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Try markdown code block with json
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start + 7..].find("```") {
            let json = trimmed[start + 7..start + 7 + end].trim();
            if json.starts_with('{') {
                return Some(json.to_string());
            }
        }
    }

    // Try bare code block
    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed[start + 3..].find("```") {
            let json = trimmed[start + 3..start + 3 + end].trim();
            if json.starts_with('{') {
                return Some(json.to_string());
            }
        }
    }

    // Find first { and last }
    let first_brace = trimmed.find('{')?;
    let last_brace = trimmed.rfind('}')?;
    if last_brace > first_brace {
        return Some(trimmed[first_brace..=last_brace].to_string());
    }

    None
}

/// Parse raw LLM output into a validated response
pub fn parse_specialist_output(raw: &str) -> ParseOutcome {
    // Step 1: Extract JSON
    let json_str = match extract_json(raw) {
        Some(j) => j,
        None => {
            return ParseOutcome::NoJson {
                raw: truncate(raw, 500),
            }
        }
    };

    // Step 2: Parse JSON
    let response: UnifiedSpecialistResponse = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(e) => {
            return ParseOutcome::InvalidJson {
                raw: truncate(&json_str, 500),
                error: e.to_string(),
            }
        }
    };

    // Step 3: Validate schema
    let errors = response.validate();
    if !errors.is_empty() {
        return ParseOutcome::SchemaError { response, errors };
    }

    ParseOutcome::Success(response)
}

/// Create a timeout parse outcome
pub fn timeout_outcome(elapsed_secs: u64) -> ParseOutcome {
    ParseOutcome::Timeout { elapsed_secs }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_clean() {
        let raw = r#"{"can_answer": true, "confidence": 0.9}"#;
        let json = extract_json(raw).unwrap();
        assert!(json.starts_with('{'));
    }

    #[test]
    fn test_extract_json_markdown() {
        let raw = r#"Here is the response:
```json
{"can_answer": true, "confidence": 0.9}
```"#;
        let json = extract_json(raw).unwrap();
        assert!(json.contains("can_answer"));
    }

    #[test]
    fn test_extract_json_with_prose() {
        let raw =
            r#"I analyzed the data. {"can_answer": true, "confidence": 0.9} That's my answer."#;
        let json = extract_json(raw).unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_parse_valid_response() {
        let raw = r#"{"can_answer": true, "confidence": 0.85, "diagnosis": "All good"}"#;
        let outcome = parse_specialist_output(raw);
        assert!(outcome.is_success());
    }

    #[test]
    fn test_parse_no_json() {
        let raw = "This is just prose without any JSON";
        let outcome = parse_specialist_output(raw);
        assert_eq!(outcome.error_kind(), "no_json");
    }
}
