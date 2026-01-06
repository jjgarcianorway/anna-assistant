//! Query translation - converts natural language to structured queries.
//! v0.0.825: Improved JSON parsing robustness for malformed LLM responses.

use std::collections::HashMap;
use tracing::{debug, warn};

use crate::ollama;
use super::types::ParsedQuery;

/// Translate natural language query to structured form
pub async fn translate_query(model: &str, query: &str) -> Result<ParsedQuery, String> {
    let prompt = format!(
        r#"Analyze this Linux system query and extract:
1. intent: what the user wants (e.g., "check_ram", "list_services", "configure_vim")
2. domain: category (system, network, storage, services, desktop, security)
3. probes: shell commands needed (e.g., ["free -h", "cat /proc/meminfo"])

Query: "{}"

Respond ONLY with valid JSON, no other text:
{{"intent": "...", "domain": "...", "probes": ["...", "..."]}}"#,
        query
    );

    let response = ollama::chat_with_timeout(model, &prompt, 10)
        .await
        .map_err(|e| format!("Translator failed: {}", e))?;

    // Parse JSON response with robust error handling
    let json = parse_json_response(&response)
        .map_err(|e| format!("Failed to parse translator response: {} - Raw: {}", e, response))?;

    Ok(ParsedQuery {
        intent: json["intent"].as_str().unwrap_or("unknown").to_string(),
        domain: json["domain"].as_str().unwrap_or("system").to_string(),
        probes: json["probes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        entities: HashMap::new(),
    })
}

/// v0.0.825: Robust JSON parsing that handles common LLM malformed responses
fn parse_json_response(response: &str) -> Result<serde_json::Value, String> {
    // Try direct parse first
    if let Ok(json) = serde_json::from_str(response) {
        return Ok(json);
    }

    // Try to extract JSON from response
    let start = response.find('{').ok_or("No JSON object found")?;
    let end = response.rfind('}').ok_or("No closing brace found")?;

    if end <= start {
        return Err("Invalid JSON structure".to_string());
    }

    let json_str = &response[start..=end];

    // Try parsing extracted JSON
    if let Ok(json) = serde_json::from_str(json_str) {
        return Ok(json);
    }

    // v0.0.825: Try to fix common LLM JSON errors
    let fixed = fix_common_json_errors(json_str);
    debug!("Attempting to parse fixed JSON: {}", fixed);

    serde_json::from_str(&fixed).map_err(|e| format!("JSON parse error: {}", e))
}

/// v0.0.825: Fix common JSON errors from LLM responses
fn fix_common_json_errors(json_str: &str) -> String {
    let mut fixed = json_str.to_string();

    // Fix: extra closing bracket before brace like `])"}`
    fixed = fixed.replace("])", "]");
    fixed = fixed.replace(")}", "}");
    fixed = fixed.replace("]\"}", "]}");

    // Fix: missing closing bracket in arrays
    // Count brackets to ensure they're balanced
    let open_brackets = fixed.matches('[').count();
    let close_brackets = fixed.matches(']').count();
    if open_brackets > close_brackets {
        // Find position before last }
        if let Some(pos) = fixed.rfind('}') {
            let missing = open_brackets - close_brackets;
            let brackets: String = (0..missing).map(|_| ']').collect();
            fixed.insert_str(pos, &brackets);
        }
    }

    // Fix: trailing comma before closing bracket/brace
    fixed = fixed.replace(",]", "]");
    fixed = fixed.replace(",}", "}");

    // Fix: unquoted string values (common with some models)
    // This is a simplistic fix - just ensure basic structure

    // Fix: double quotes inside strings (escape them)
    // This is tricky - we'd need proper parsing

    fixed
}

/// Extract tags for knowledge lookup from parsed query
pub fn extract_knowledge_tags(parsed: &ParsedQuery) -> Vec<String> {
    let mut tags = vec![];

    // Add intent keywords
    for part in parsed.intent.split('_') {
        if part.len() > 2 {
            tags.push(part.to_string());
        }
    }

    // Add domain as a tag
    tags.push(parsed.domain.clone());

    // Add any entities
    for (_, value) in &parsed.entities {
        tags.push(value.clone());
    }

    // Deduplicate
    tags.sort();
    tags.dedup();
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_knowledge_tags() {
        let mut entities = HashMap::new();
        entities.insert("service".to_string(), "nginx".to_string());

        let parsed = ParsedQuery {
            intent: "check_ram_usage".to_string(),
            domain: "system".to_string(),
            probes: vec![],
            entities,
        };

        let tags = extract_knowledge_tags(&parsed);
        assert!(tags.contains(&"check".to_string()));
        assert!(tags.contains(&"ram".to_string()));
        assert!(tags.contains(&"usage".to_string()));
        assert!(tags.contains(&"system".to_string()));
        assert!(tags.contains(&"nginx".to_string()));
    }

    #[test]
    fn test_parse_json_response_valid() {
        let response = r#"{"intent": "check_ram", "domain": "system", "probes": ["free -h"]}"#;
        let result = parse_json_response(response);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["intent"], "check_ram");
    }

    #[test]
    fn test_parse_json_response_with_extra_text() {
        let response = r#"Here is the JSON: {"intent": "check_disk", "domain": "storage", "probes": ["df -h"]} That's it."#;
        let result = parse_json_response(response);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["intent"], "check_disk");
    }

    #[test]
    fn test_parse_json_response_malformed_extra_bracket() {
        // This is the actual error from the user's session
        let response = r#"{"intent": "check_updates", "domain": "system", "probes": ["sudo apt list --upgradable", "yum check"])"}"#;
        let result = parse_json_response(response);
        assert!(result.is_ok(), "Should handle extra ) before }}");
        let json = result.unwrap();
        assert_eq!(json["intent"], "check_updates");
    }

    #[test]
    fn test_parse_json_response_trailing_comma() {
        let response = r#"{"intent": "list_services", "domain": "services", "probes": ["systemctl list-units",]}"#;
        let result = parse_json_response(response);
        assert!(result.is_ok(), "Should handle trailing comma");
    }

    #[test]
    fn test_fix_common_json_errors() {
        // Extra ) before }
        let fixed = fix_common_json_errors(r#"{"a": [1, 2])}"#);
        assert_eq!(fixed, r#"{"a": [1, 2]}"#);

        // Trailing comma
        let fixed = fix_common_json_errors(r#"{"a": [1, 2,]}"#);
        assert_eq!(fixed, r#"{"a": [1, 2]}"#);

        // Missing closing bracket
        let fixed = fix_common_json_errors(r#"{"a": [1, 2}"#);
        assert_eq!(fixed, r#"{"a": [1, 2]}"#);
    }
}
