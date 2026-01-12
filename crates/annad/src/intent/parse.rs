//! JSON parsing utilities for intent classification responses.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use anyhow::Result;
use tracing::warn;

use super::fallback::fallback_understanding;

/// Parse quick classification response
pub fn parse_quick_response(response: &str, original_question: &str) -> Result<DeepUnderstanding> {
    let json_str = extract_json_from_response(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let category = match parsed.get("category").and_then(|c| c.as_str()) {
            Some("FACTUAL") => IntentCategory::Factual,
            Some("HOWTO") => IntentCategory::HowTo,
            Some("TROUBLESHOOT") => IntentCategory::Troubleshoot,
            Some("MULTI") => IntentCategory::Multi,
            Some("UNCLEAR") => IntentCategory::Unclear,
            _ => IntentCategory::Factual,
        };

        let confidence = parsed
            .get("confidence")
            .and_then(|c| c.as_f64())
            .map(|c| c as f32)
            .unwrap_or(0.7);

        let interpreted_as = parsed
            .get("interpreted_as")
            .and_then(|s| s.as_str())
            .unwrap_or(original_question)
            .to_string();

        let entities = extract_string_array(&parsed, "entities");
        let topic = parsed
            .get("topic")
            .and_then(|t| t.as_str())
            .filter(|t| !t.is_empty() && *t != "null")
            .map(String::from);

        // v0.0.990: Use the improved should_ask_confirmation for quick responses too
        let needs_confirmation = super::classify::should_ask_confirmation(
            confidence,
            &[],  // No missing_info in quick classify
            &[],  // No ambiguities in quick classify
            &category,
            original_question,
        );

        return Ok(DeepUnderstanding {
            interpreted_as,
            required_info: vec![],
            missing_info: vec![],
            ambiguities: vec![],
            confidence,
            category,
            entities,
            topic,
            sub_questions: None,
            clarification_needed: None,
            needs_confirmation,
            suggested_commands: vec![],
        });
    }

    // Parsing failed - return low confidence to trigger deep understanding
    Ok(DeepUnderstanding {
        interpreted_as: original_question.to_string(),
        confidence: 0.3,
        category: IntentCategory::Factual,
        ..Default::default()
    })
}

/// Parse the LLM's chain-of-thought response into DeepUnderstanding
pub fn parse_understanding_response(
    response: &str,
    original_question: &str,
) -> Result<DeepUnderstanding> {
    let json_str = extract_json_from_response(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let category = match parsed.get("category").and_then(|c| c.as_str()) {
            Some("FACTUAL") => IntentCategory::Factual,
            Some("HOWTO") => IntentCategory::HowTo,
            Some("TROUBLESHOOT") => IntentCategory::Troubleshoot,
            Some("MULTI") => IntentCategory::Multi,
            Some("UNCLEAR") => IntentCategory::Unclear,
            _ => IntentCategory::Factual,
        };

        let confidence = parsed
            .get("confidence")
            .and_then(|c| c.as_f64())
            .map(|c| c as f32)
            .unwrap_or(0.5);

        let interpreted_as = parsed
            .get("interpreted_as")
            .and_then(|s| s.as_str())
            .unwrap_or(original_question)
            .to_string();

        let required_info = extract_string_array(&parsed, "required_info");
        let missing_info = extract_string_array(&parsed, "missing_info");
        let ambiguities = extract_string_array(&parsed, "ambiguities");
        let entities = extract_string_array(&parsed, "entities");

        let topic = parsed
            .get("topic")
            .and_then(|t| t.as_str())
            .filter(|t| !t.is_empty() && *t != "null")
            .map(String::from);

        let sub_questions = parsed
            .get("sub_questions")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let clarification_needed = parsed
            .get("clarification_needed")
            .and_then(|c| c.as_str())
            .filter(|s| !s.is_empty() && *s != "null")
            .map(String::from);

        let needs_confirmation = super::classify::should_ask_confirmation(
            confidence,
            &missing_info,
            &ambiguities,
            &category,
            original_question,
        );

        return Ok(DeepUnderstanding {
            interpreted_as,
            required_info,
            missing_info,
            ambiguities,
            confidence,
            category,
            entities,
            topic,
            sub_questions,
            clarification_needed,
            needs_confirmation,
            suggested_commands: vec![],
        });
    }

    warn!("Failed to parse understanding JSON, using fallback");
    Ok(fallback_understanding(original_question))
}

/// Extract JSON from a response that may contain reasoning text
/// v0.0.896: Robust extraction with proper brace matching
pub fn extract_json_from_response(response: &str) -> String {
    let response = response.trim();

    // First try: Clean markdown code blocks
    let cleaned = response
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Find all potential JSON start positions and try each until one parses
    for (start_idx, _) in cleaned.char_indices().filter(|(_, c)| *c == '{') {
        if let Some(json_str) = extract_balanced_json(&cleaned[start_idx..]) {
            if serde_json::from_str::<serde_json::Value>(&json_str).is_ok() {
                return json_str;
            }
        }
    }

    cleaned.to_string()
}

/// Extract a balanced JSON object by counting braces
fn extract_balanced_json(s: &str) -> Option<String> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut end_idx = None;

    for (idx, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }

    end_idx.map(|end| s[..=end].to_string())
}

/// Extract a string array from JSON
pub fn extract_string_array(parsed: &serde_json::Value, key: &str) -> Vec<String> {
    parsed
        .get(key)
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract JSON array from response
pub fn extract_json_array_from_response(response: &str) -> String {
    let response = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if let Some(start) = response.find('[') {
        if let Some(end) = response.rfind(']') {
            return response[start..=end].to_string();
        }
    }

    "[]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_response() {
        // Test with reasoning text before JSON
        let response = "Let me think...\n\n{\"category\":\"FACTUAL\",\"confidence\":0.9}";
        let json = extract_json_from_response(response);
        assert!(json.contains("FACTUAL"));

        // Test with markdown code block
        let response = "```json\n{\"category\":\"HOWTO\"}\n```";
        let json = extract_json_from_response(response);
        assert!(json.contains("HOWTO"));

        // Test nested objects
        let response = r#"Analysis: {"outer": {"inner": "value"}, "category": "TEST"}"#;
        let json = extract_json_from_response(response);
        assert!(json.contains("inner"));

        // Test with braces inside strings
        let response = r#"{"message": "Use {braces} in config", "ok": true}"#;
        let json = extract_json_from_response(response);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["ok"], true);
    }

    #[test]
    fn test_parse_quick_response() {
        let json = r#"{"interpreted_as":"check disk","category":"FACTUAL","confidence":0.9,"entities":["disk"]}"#;
        let result = parse_quick_response(json, "test").unwrap();
        assert_eq!(result.interpreted_as, "check disk");
        assert_eq!(result.category, IntentCategory::Factual);
    }
}
