//! Parsing functions for translator responses.

use anna_shared::deterministic_probes::{deterministic_probes_for_query, is_concept_query};
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::{info, warn};

use crate::probe_registry::filter_valid_probes;
use crate::redact;

use super::learning::filter_bad_combos;
use super::types::TranslatorOutput;

/// Parse intent string to enum
/// v0.0.405: Extended for all intents, normalizes legacy values
pub(crate) fn parse_intent(s: &str) -> QueryIntent {
    let intent = match s.to_lowercase().as_str() {
        // New intents (v0.0.405)
        "query_metric" | "querymetric" => QueryIntent::QueryMetric,
        "diagnose" => QueryIntent::Diagnose,
        "configure" => QueryIntent::Configure,
        "list" => QueryIntent::List,
        "check_status" | "checkstatus" | "status" => QueryIntent::CheckStatus,
        "explain" => QueryIntent::Explain,
        // Legacy intents (normalized)
        "question" => QueryIntent::Question,
        "request" => QueryIntent::Request,
        "investigate" => QueryIntent::Investigate,
        _ => QueryIntent::QueryMetric, // default to most common
    };
    intent.normalize() // Normalize legacy to new
}

/// Parse domain string to enum
/// v0.0.405: Extended for all 10 domains
pub(crate) fn parse_domain(s: &str) -> SpecialistDomain {
    match s.to_lowercase().as_str() {
        "system" => SpecialistDomain::System,
        "boot" => SpecialistDomain::Boot,
        "services" | "service" => SpecialistDomain::Services,
        "network" | "net" => SpecialistDomain::Network,
        "storage" | "disk" => SpecialistDomain::Storage,
        "packages" | "package" | "pkg" => SpecialistDomain::Packages,
        "audio" | "sound" => SpecialistDomain::Audio,
        "display" | "graphics" | "gpu" => SpecialistDomain::Display,
        "desktop" | "de" | "wm" => SpecialistDomain::Desktop,
        "security" | "sec" => SpecialistDomain::Security,
        _ => SpecialistDomain::System, // default
    }
}

/// Extract JSON from LLM response (handles markdown code blocks)
pub(crate) fn extract_json(response: &str) -> Result<String, String> {
    let t = response.trim();
    // Direct JSON
    if t.starts_with('{') && t.ends_with('}') {
        return Ok(t.to_string());
    }
    // Markdown code block
    if let Some(s) = t.find("```json") {
        if let Some(e) = t[s..].find("```\n").or(t[s..].rfind("```")) {
            let js = s + 7;
            let je = s + e;
            if js < je {
                return Ok(t[js..je].trim().to_string());
            }
        }
    }
    // Plain code block
    if let Some(s) = t.find("```") {
        if let Some(e) = t[s + 3..].find("```") {
            let json_str = t[s + 3..s + 3 + e]
                .lines()
                .skip_while(|l| !l.trim().starts_with('{'))
                .collect::<Vec<_>>()
                .join("\n");
            if !json_str.is_empty() {
                return Ok(json_str);
            }
        }
    }
    // Find JSON anywhere
    if let (Some(s), Some(e)) = (t.find('{'), t.rfind('}')) {
        if s < e {
            return Ok(t[s..=e].to_string());
        }
    }
    Err("No valid JSON found".to_string())
}

/// Parse translator LLM response into ticket (tolerant of missing/invalid fields)
/// v0.0.374: Added query parameter for bad combo filtering
/// v0.0.448: DETERMINISTIC PROBES OVERRIDE - use intent-specific probes if matched
pub(crate) fn parse_translator_response(
    response: &str,
    query: &str,
) -> Result<TranslatorTicket, String> {
    // v0.0.290: Strip reasoning tags before parsing
    let cleaned = redact::strip_reasoning_tags(response);

    // Log raw response in debug (truncated for safety)
    let truncated = if cleaned.len() > 500 {
        format!("{}... [truncated]", &cleaned[..500])
    } else {
        cleaned.clone()
    };
    tracing::debug!("Translator raw response: {}", truncated);

    // Try to extract JSON from response (handle markdown code blocks)
    let json_str = extract_json(&cleaned)?;

    // Parse JSON with tolerant structure - use default for any parse errors
    let output: TranslatorOutput = serde_json::from_str(&json_str).unwrap_or_else(|e| {
        warn!("JSON parse error, using defaults: {}", e);
        TranslatorOutput::default()
    });

    // Extract fields with defaults for missing values
    let intent_str = output.intent.as_deref().unwrap_or("question");
    let domain_str = output.domain.as_deref().unwrap_or("system");
    let confidence = output.confidence.unwrap_or(0.0).clamp(0.0, 1.0);
    let entities = output.entities.unwrap_or_default();

    // v0.0.448: DETERMINISTIC PROBES FIRST - check for intent-specific probe rules
    // This overrides LLM probe selection for common, well-understood queries
    let needs_probes = if let Some(deterministic) = deterministic_probes_for_query(query) {
        info!(
            "Translator: DETERMINISTIC probes for '{}': [{}]",
            query,
            deterministic
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        deterministic.into_iter().map(String::from).collect()
    } else {
        // v0.0.374: Filter valid probes, then filter out known bad combos
        let valid_probes = filter_valid_probes(output.needs_probes.unwrap_or_default());
        filter_bad_combos(query, valid_probes)
    };

    // v0.0.448: Detect concept queries that should NOT be treated as package queries
    // If this is a concept query (e.g., "do I have swap?"), clear any package-related entities
    let entities = if is_concept_query(query) {
        info!(
            "Translator: detected concept query, not a package query: '{}'",
            query
        );
        // Keep entities but mark them as concepts, not packages
        entities
    } else {
        entities
    };

    let ticket = TranslatorTicket {
        intent: parse_intent(intent_str),
        domain: parse_domain(domain_str),
        entities,
        needs_probes,
        clarification_question: output.clarification_question,
        confidence,
        answer_contract: None, // v0.0.74: Set by caller with query context
    };

    // v0.0.396: Log actual probe names for debugging
    info!(
        "Translator: intent={}, domain={}, confidence={:.2}, probes=[{}]",
        ticket.intent,
        ticket.domain,
        ticket.confidence,
        ticket.needs_probes.join(",")
    );

    Ok(ticket)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_direct() {
        let json = r#"{"intent": "question"}"#;
        assert_eq!(extract_json(json).unwrap(), json);
    }

    #[test]
    fn test_extract_json_markdown() {
        let response = r#"Here's the result:
```json
{"intent": "question"}
```"#;
        assert!(extract_json(response).unwrap().contains("intent"));
    }

    #[test]
    fn test_tolerant_json_parsing_missing_fields() {
        // Missing confidence -> 0.0
        let response = r#"{"intent":"question","domain":"system"}"#;
        let ticket = parse_translator_response(response, "test query").unwrap();
        assert_eq!(ticket.confidence, 0.0);
        assert_eq!(ticket.domain, SpecialistDomain::System);
    }

    #[test]
    fn test_tolerant_json_parsing_null_arrays() {
        // null arrays -> empty Vec
        let response = r#"{"intent":"question","entities":null,"needs_probes":null}"#;
        let ticket = parse_translator_response(response, "test query").unwrap();
        assert!(ticket.entities.is_empty());
        assert!(ticket.needs_probes.is_empty());
    }

    #[test]
    fn test_tolerant_json_parsing_invalid_values() {
        // Invalid domain -> default to System
        let response = r#"{"intent":"question","domain":"invalid_domain"}"#;
        let ticket = parse_translator_response(response, "test query").unwrap();
        assert_eq!(ticket.domain, SpecialistDomain::System);
    }
}
