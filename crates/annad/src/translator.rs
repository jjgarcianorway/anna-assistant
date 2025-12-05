//! LLM-based translator for query classification.
//!
//! Converts user text to structured TranslatorTicket JSON.

use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::ollama;

/// Probe IDs for translator to select from
pub const PROBE_IDS: &[&str] = &[
    "top_memory",      // ps aux --sort=-%mem
    "top_cpu",         // ps aux --sort=-%cpu
    "cpu_info",        // lscpu
    "memory_info",     // free -h
    "disk_usage",      // df -h
    "block_devices",   // lsblk
    "network_addrs",   // ip addr show
    "network_routes",  // ip route
    "listening_ports", // ss -tulpn
    "failed_services", // systemctl --failed
    "system_logs",     // journalctl -p warning..alert -n 200 --no-pager
];

/// Map probe ID to actual command
pub fn probe_id_to_command(id: &str) -> Option<&'static str> {
    match id {
        "top_memory" => Some("ps aux --sort=-%mem"),
        "top_cpu" => Some("ps aux --sort=-%cpu"),
        "cpu_info" => Some("lscpu"),
        "memory_info" => Some("free -h"),
        "disk_usage" => Some("df -h"),
        "block_devices" => Some("lsblk"),
        "network_addrs" => Some("ip addr show"),
        "network_routes" => Some("ip route"),
        "listening_ports" => Some("ss -tulpn"),
        "failed_services" => Some("systemctl --failed"),
        "system_logs" => Some("journalctl -p warning..alert -n 200 --no-pager"),
        _ => None,
    }
}

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
        let hw_summary = format!("CPU cores: {}, RAM: {:.0}GB, GPU: {}", cpu_cores, ram_gb, gpu_str);
        Self {
            query: query.to_string(),
            hw_summary,
        }
    }
}

/// Build the translator system prompt - strict enum constraints
fn build_translator_prompt() -> String {
    format!(
        r#"Classify query. Output ONLY valid JSON matching this exact schema:
{{"intent":"question|request|investigate","domain":"system|network|storage|security|packages","entities":[],"needs_probes":[],"clarification_question":null,"confidence":0.0-1.0}}

STRICT RULES:
- intent MUST be exactly one of: question, request, investigate
- domain MUST be exactly one of: system, network, storage, security, packages
- needs_probes MUST only contain IDs from: {}
- confidence MUST be a decimal 0.0-1.0
- Set clarification_question if query is ambiguous
- Select 1-3 probes maximum
Output raw JSON only. No markdown. No explanation."#,
        PROBE_IDS.join(", ")
    )
}

/// Build minimal translator request (< 2KB)
pub fn build_translator_request(input: &TranslatorInput) -> String {
    let prompt = build_translator_prompt();
    format!("{}\nHW: {}\nQuery: {}", prompt, input.hw_summary, input.query)
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

/// Filter probe IDs to only valid ones
fn filter_valid_probes(probes: Vec<String>) -> Vec<String> {
    probes
        .into_iter()
        .filter(|p| PROBE_IDS.contains(&p.as_str()))
        .collect()
}

/// Translate user query to structured ticket using LLM (with minimal input)
pub async fn translate_with_context(
    model: &str,
    input: &TranslatorInput,
    timeout_secs: u64,
) -> Result<TranslatorTicket, String> {
    let full_prompt = build_translator_request(input);

    info!(
        "Translator: processing query (payload {} bytes)",
        full_prompt.len()
    );

    let response = ollama::chat_with_timeout(model, &full_prompt, timeout_secs)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;

    parse_translator_response(&response)
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

    parse_translator_response(&response)
}

/// Parse translator LLM response into ticket (tolerant of missing/invalid fields)
fn parse_translator_response(response: &str) -> Result<TranslatorTicket, String> {
    // Log raw response in debug (truncated for safety)
    let truncated = if response.len() > 500 {
        format!("{}... [truncated]", &response[..500])
    } else {
        response.to_string()
    };
    tracing::debug!("Translator raw response: {}", truncated);

    // Try to extract JSON from response (handle markdown code blocks)
    let json_str = extract_json(response)?;

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
    let needs_probes = filter_valid_probes(output.needs_probes.unwrap_or_default());

    let ticket = TranslatorTicket {
        intent: parse_intent(intent_str),
        domain: parse_domain(domain_str),
        entities,
        needs_probes,
        clarification_question: output.clarification_question,
        confidence,
    };

    info!(
        "Translator: intent={}, domain={}, confidence={:.2}, probes={}",
        ticket.intent,
        ticket.domain,
        ticket.confidence,
        ticket.needs_probes.len()
    );

    Ok(ticket)
}

/// Extract JSON from LLM response (handles markdown code blocks)
fn extract_json(response: &str) -> Result<String, String> {
    let t = response.trim();
    // Direct JSON
    if t.starts_with('{') && t.ends_with('}') { return Ok(t.to_string()); }
    // Markdown code block
    if let Some(s) = t.find("```json") {
        if let Some(e) = t[s..].find("```\n").or(t[s..].rfind("```")) {
            let js = s + 7; let je = s + e;
            if js < je { return Ok(t[js..je].trim().to_string()); }
        }
    }
    // Plain code block
    if let Some(s) = t.find("```") {
        if let Some(e) = t[s+3..].find("```") {
            let json_str = t[s+3..s+3+e].lines()
                .skip_while(|l| !l.trim().starts_with('{'))
                .collect::<Vec<_>>().join("\n");
            if !json_str.is_empty() { return Ok(json_str); }
        }
    }
    // Find JSON anywhere
    if let (Some(s), Some(e)) = (t.find('{'), t.rfind('}')) {
        if s < e { return Ok(t[s..=e].to_string()); }
    }
    Err("No valid JSON found".to_string())
}

/// Fallback keyword-based translation (used when LLM fails)
pub fn translate_fallback(query: &str) -> TranslatorTicket {
    warn!("Using fallback keyword translator");
    let q = query.to_lowercase();

    let domain = if q.contains("network") || q.contains("ip ") || q.contains("interface") || q.contains("dns") || q.contains("port") || q.contains("route") {
        SpecialistDomain::Network
    } else if q.contains("disk") || q.contains("storage") || q.contains("space") || q.contains("mount") || q.contains("partition") {
        SpecialistDomain::Storage
    } else if q.contains("security") || q.contains("firewall") || q.contains("permission") || q.contains("ssh") {
        SpecialistDomain::Security
    } else if q.contains("package") || q.contains("install") || q.contains("pacman") || q.contains("apt") {
        SpecialistDomain::Packages
    } else {
        SpecialistDomain::System
    };

    let intent = if q.contains("install") || q.contains("start") || q.contains("stop") || q.contains("configure") {
        QueryIntent::Request
    } else if q.contains("why") || q.contains("debug") || q.contains("fix") {
        QueryIntent::Investigate
    } else { QueryIntent::Question };

    let mut needs_probes = Vec::new();
    if q.contains("memory") || q.contains("ram") { needs_probes.extend(["top_memory", "memory_info"].map(String::from)); }
    if q.contains("cpu") { needs_probes.extend(["top_cpu", "cpu_info"].map(String::from)); }
    if q.contains("disk") || q.contains("space") { needs_probes.push("disk_usage".to_string()); }
    if q.contains("network") || q.contains("ip") { needs_probes.push("network_addrs".to_string()); }
    if q.contains("port") || q.contains("listen") { needs_probes.push("listening_ports".to_string()); }

    TranslatorTicket { intent, domain, entities: Vec::new(), needs_probes, clarification_question: None, confidence: 0.3 }
}

/// Maximum allowed translator payload size (8KB)
pub const MAX_TRANSLATOR_PAYLOAD_SIZE: usize = 8192;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_id_to_command() {
        assert_eq!(
            probe_id_to_command("top_memory"),
            Some("ps aux --sort=-%mem")
        );
        assert_eq!(probe_id_to_command("invalid"), None);
    }

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
    fn test_filter_valid_probes() {
        let probes = vec![
            "top_memory".to_string(),
            "invalid".to_string(),
            "cpu_info".to_string(),
        ];
        let filtered = filter_valid_probes(probes);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"top_memory".to_string()));
        assert!(!filtered.contains(&"invalid".to_string()));
    }

    #[test]
    fn test_fallback_domain_classification() {
        let ticket = translate_fallback("show me memory usage");
        assert_eq!(ticket.domain, SpecialistDomain::System);
        assert!(ticket.needs_probes.contains(&"top_memory".to_string()));

        let ticket = translate_fallback("check network interfaces");
        assert_eq!(ticket.domain, SpecialistDomain::Network);
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
        let ticket = parse_translator_response(response).unwrap();
        assert_eq!(ticket.confidence, 0.0);
        assert_eq!(ticket.domain, SpecialistDomain::System);
    }

    #[test]
    fn test_tolerant_json_parsing_null_arrays() {
        // null arrays -> empty Vec
        let response = r#"{"intent":"question","entities":null,"needs_probes":null}"#;
        let ticket = parse_translator_response(response).unwrap();
        assert!(ticket.entities.is_empty());
        assert!(ticket.needs_probes.is_empty());
    }

    #[test]
    fn test_tolerant_json_parsing_invalid_values() {
        // Invalid domain -> default to System
        let response = r#"{"intent":"question","domain":"invalid_domain"}"#;
        let ticket = parse_translator_response(response).unwrap();
        assert_eq!(ticket.domain, SpecialistDomain::System);
    }
}
