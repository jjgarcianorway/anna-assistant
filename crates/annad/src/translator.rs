//! LLM-based translator for query classification (v0.0.374).
//!
//! Converts user text to structured TranslatorTicket JSON.
//! v0.0.74: Now includes AnswerContract for answer shaping.
//! v0.0.164: Probe registry extracted to separate module.
//! v0.0.290: Strip reasoning tags from translator responses.
//! v0.0.318: Added TranslatorResult with debug info for LLM call visibility.
//! v0.0.322: Integrated probe learning - recommends probes based on past effectiveness.
//! v0.0.327: Uses load_with_decay() for automatic learning decay.
//! v0.0.333: Only uses learning when confidence is sufficient.
//! v0.0.374: Filter out probe combinations known to fail for similar queries.

use anna_shared::answer_contract::AnswerContract;
use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, warn};

use crate::ollama;
use crate::redact;

/// v0.0.318: Translator result with debug info for LLM call visibility
#[derive(Debug, Clone)]
pub struct TranslatorResult {
    /// The parsed ticket
    pub ticket: TranslatorTicket,
    /// The full prompt sent to the LLM
    pub prompt: String,
    /// The raw response from the LLM
    pub response: String,
    /// Duration of the LLM call in milliseconds
    pub duration_ms: u64,
}

// Re-export probe registry for backwards compatibility
pub use crate::probe_registry::{filter_valid_probes, probe_id_to_command, PROBE_IDS};

// Re-export fallback translator for backwards compatibility
pub use crate::translator_fallback::translate_fallback;

/// Internal JSON structure for LLM output parsing (tolerant of missing fields)
#[derive(Debug, Serialize, Deserialize, Default)]
struct TranslatorOutput {
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    entities: Option<Vec<String>>,
    #[serde(default)]
    needs_probes: Option<Vec<String>>,
    #[serde(default)]
    clarification_question: Option<String>,
    #[serde(default)]
    confidence: Option<f32>,
}

/// Minimal translator input - keeps payload small for fast inference
#[derive(Debug, Clone)]
pub struct TranslatorInput {
    pub query: String,
    pub hw_summary: String, // one line: "CPU cores: 8, RAM: 16GB, GPU: none"
}

impl TranslatorInput {
    /// Create minimal input for translator
    pub fn new(query: &str, cpu_cores: u32, ram_gb: f64, has_gpu: bool) -> Self {
        let gpu_str = if has_gpu { "yes" } else { "none" };
        let hw_summary = format!(
            "CPU cores: {}, RAM: {:.0}GB, GPU: {}",
            cpu_cores, ram_gb, gpu_str
        );
        Self {
            query: query.to_string(),
            hw_summary,
        }
    }
}

/// Build the translator system prompt - strict enum constraints
/// v0.0.391: Improved with more examples for common query types
/// v0.0.396: Stronger domain classification rules
fn build_translator_prompt() -> String {
    format!(
        r#"Classify query into domain and probes. Output ONLY valid JSON.

DOMAIN RULES (use EXACTLY these):
- STORAGE: disk, space, folders, directories, storage, du, df, filesystem, mount
- SYSTEM: CPU, RAM, memory, cores, processes, services, errors, logs, boot
- NETWORK: IP, DNS, ports, connections, interfaces, routing
- PACKAGES: install, package, updates, apt, pacman, pip
- SECURITY: firewall, permissions, users, groups, ssh

JSON schema:
{{"intent":"question|request|investigate","domain":"system|network|storage|security|packages","entities":[],"needs_probes":[],"clarification_question":null,"confidence":0.0-1.0}}

PROBE MAPPING (use EXACTLY these probe IDs):
- folders/directories taking space → domain=storage, probes=["largest_dirs","largest_home"]
- disk space/usage → domain=storage, probes=["df"]
- RAM/memory → domain=system, probes=["memory_info"]
- CPU info → domain=system, probes=["cpu_info"]
- system health → domain=system, probes=["memory_info","disk_usage","failed_services"]
- installed packages → domain=packages, probes=["installed_packages"]

Available probe IDs: {}

RULES:
- Select 1-3 probes maximum
- Output raw JSON only, no markdown, no explanation"#,
        PROBE_IDS.join(", ")
    )
}

/// Build minimal translator request (< 2KB)
pub fn build_translator_request(input: &TranslatorInput) -> String {
    let prompt = build_translator_prompt();

    // v0.0.322: Add learned probe recommendations if available
    let recommendations = get_probe_recommendations(&input.query);

    if recommendations.is_empty() {
        format!(
            "{}\nHW: {}\nQuery: {}",
            prompt, input.hw_summary, input.query
        )
    } else {
        format!(
            "{}\nHW: {}\nLearned: For this type of query, effective probes have been: {}\nQuery: {}",
            prompt, input.hw_summary, recommendations, input.query
        )
    }
}

/// v0.0.322: Get probe recommendations from learning store
/// v0.0.325: Also uses keyword-based suggestions
/// v0.0.327: Uses load_with_decay() for automatic decay
/// v0.0.333: Only returns recommendations if learning confidence is sufficient
fn get_probe_recommendations(query: &str) -> String {
    let store = ProbeLearningStore::load_with_decay();

    // v0.0.333: Check if we should trust the learning data
    if !store.should_use_learning() {
        info!("Learning confidence too low ({:.0}%), skipping recommendations",
              store.confidence_factor() * 100.0);
        return String::new();
    }

    let category = QueryCategory::from_query(query);

    // Get category-based recommendations
    let category_recs = store.get_recommended_probes(&category);

    // v0.0.325: Get keyword-based suggestions
    let keyword_suggestions = store.suggest_probes_for_query(query);

    // Combine both sources, prioritizing keyword matches
    let mut combined: std::collections::HashMap<String, f32> = std::collections::HashMap::new();

    // Add category recommendations (threshold based on confidence)
    let score_threshold = 0.5 + (store.confidence_factor() * 0.2); // 0.5-0.7 based on confidence
    for (probe_id, score) in &category_recs {
        if *score > score_threshold {
            combined.insert(probe_id.clone(), *score);
        }
    }

    // Boost probes that also match keywords
    for (probe_id, keyword_count) in &keyword_suggestions {
        let boost = (*keyword_count as f32 * 0.1).min(0.3); // Max 30% boost
        let entry = combined.entry(probe_id.clone()).or_insert(0.5);
        *entry = (*entry + boost).min(1.0);
    }

    // Sort by score
    let mut sorted: Vec<_> = combined.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let good_probes: Vec<String> = sorted
        .into_iter()
        .take(5) // Top 5
        .map(|(probe_id, score)| format!("{} ({:.0}%)", probe_id, score * 100.0))
        .collect();

    if good_probes.is_empty() {
        String::new()
    } else {
        info!("Using learned probes (confidence {:.0}%): {}",
              store.confidence_factor() * 100.0, good_probes.join(", "));
        good_probes.join(", ")
    }
}

/// Parse intent string to enum
fn parse_intent(s: &str) -> QueryIntent {
    match s.to_lowercase().as_str() {
        "question" => QueryIntent::Question,
        "request" => QueryIntent::Request,
        "investigate" => QueryIntent::Investigate,
        _ => QueryIntent::Question, // default
    }
}

/// Parse domain string to enum
fn parse_domain(s: &str) -> SpecialistDomain {
    match s.to_lowercase().as_str() {
        "system" => SpecialistDomain::System,
        "network" => SpecialistDomain::Network,
        "storage" => SpecialistDomain::Storage,
        "security" => SpecialistDomain::Security,
        "packages" => SpecialistDomain::Packages,
        _ => SpecialistDomain::System, // default
    }
}

/// Translate user query to structured ticket using LLM (with minimal input)
/// v0.0.74: Now generates AnswerContract from query for answer shaping
pub async fn translate_with_context(
    model: &str,
    input: &TranslatorInput,
    timeout_secs: u64,
) -> Result<TranslatorTicket, String> {
    let result = translate_with_debug(model, input, timeout_secs).await?;
    Ok(result.ticket)
}

/// v0.0.318: Translate with full debug info (prompt, response, timing)
pub async fn translate_with_debug(
    model: &str,
    input: &TranslatorInput,
    timeout_secs: u64,
) -> Result<TranslatorResult, String> {
    let full_prompt = build_translator_request(input);

    info!(
        "Translator: processing query (payload {} bytes)",
        full_prompt.len()
    );

    let start = Instant::now();
    let response = ollama::chat_with_timeout(model, &full_prompt, timeout_secs)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let mut ticket = parse_translator_response(&response, &input.query)?;

    // v0.0.74: Generate answer contract from original query
    ticket.answer_contract = Some(AnswerContract::from_query(&input.query));

    Ok(TranslatorResult {
        ticket,
        prompt: full_prompt,
        response,
        duration_ms,
    })
}

/// Legacy translate function (for compatibility/tests)
#[allow(dead_code)]
pub async fn translate(model: &str, query: &str) -> Result<TranslatorTicket, String> {
    // Use default hardware values for legacy calls
    let input = TranslatorInput::new(query, 4, 8.0, false);
    let full_prompt = build_translator_request(&input);

    info!("Translator: processing query");

    let response = ollama::chat(model, &full_prompt)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;

    parse_translator_response(&response, query)
}

/// v0.0.374: Filter probes that are known to fail for similar queries
fn filter_bad_combos(query: &str, probes: Vec<String>) -> Vec<String> {
    let store = ProbeLearningStore::load();
    if let Some(reason) = store.is_known_bad_combo(query, &probes) {
        // Log why we're filtering
        info!("Learning: avoiding probes due to past failure: {}", reason);
        // Remove probes that match the bad pattern
        let filtered: Vec<String> = probes
            .into_iter()
            .filter(|p| store.is_known_bad_combo(query, &[p.clone()]).is_none())
            .collect();
        if filtered.is_empty() {
            // Don't return empty - keep at least one probe
            vec!["memory_info".to_string()] // Safe default
        } else {
            filtered
        }
    } else {
        probes
    }
}

/// Parse translator LLM response into ticket (tolerant of missing/invalid fields)
/// v0.0.374: Added query parameter for bad combo filtering
fn parse_translator_response(response: &str, query: &str) -> Result<TranslatorTicket, String> {
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
    // v0.0.374: Filter valid probes, then filter out known bad combos
    let valid_probes = filter_valid_probes(output.needs_probes.unwrap_or_default());
    let needs_probes = filter_bad_combos(query, valid_probes);

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

/// Extract JSON from LLM response (handles markdown code blocks)
fn extract_json(response: &str) -> Result<String, String> {
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

/// Maximum allowed translator payload size (8KB)
#[allow(dead_code)]
pub const MAX_TRANSLATOR_PAYLOAD_SIZE: usize = 8192;

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
    fn test_translator_payload_size() {
        let input = TranslatorInput::new("what processes are using the most memory", 8, 16.0, true);
        let payload = build_translator_request(&input);
        assert!(payload.len() < MAX_TRANSLATOR_PAYLOAD_SIZE);
        assert!(payload.len() < 2048); // Should be well under 2KB
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
