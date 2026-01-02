//! Query translation - converts natural language to structured queries.

use std::collections::HashMap;
use tracing::warn;

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

    // Parse JSON response
    let json: serde_json::Value = serde_json::from_str(&response)
        .or_else(|_| {
            // Try to extract JSON from response
            if let Some(start) = response.find('{') {
                if let Some(end) = response.rfind('}') {
                    return serde_json::from_str(&response[start..=end]);
                }
            }
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No JSON found",
            )))
        })
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
}
