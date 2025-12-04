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

/// Internal JSON structure for LLM output parsing
#[derive(Debug, Serialize, Deserialize)]
struct TranslatorOutput {
    intent: String,
    domain: String,
    entities: Vec<String>,
    needs_probes: Vec<String>,
    clarification_question: Option<String>,
    confidence: f32,
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

/// Build the translator system prompt - minimal, no probe output
fn build_translator_prompt() -> String {
    format!(
        r#"Classify query. Output JSON only:
{{"intent":"<question|request|investigate>","domain":"<system|network|storage|security|packages>","entities":[],"needs_probes":[],"clarification_question":null,"confidence":0.9}}

Domains: system(CPU/memory/processes), network(IP/ports), storage(disk/mount), security(firewall/ssh), packages(apt/pacman)
Probes: {}
Rules: Select 1-3 probes max. Set clarification_question if query is vague.
JSON ONLY."#,
        PROBE_IDS.join(",")
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

/// Parse translator LLM response into ticket
fn parse_translator_response(response: &str) -> Result<TranslatorTicket, String> {

    // Try to extract JSON from response (handle markdown code blocks)
    let json_str = extract_json(&response)?;

    // Parse JSON
    let output: TranslatorOutput = serde_json::from_str(&json_str)
        .map_err(|e| format!("Invalid JSON from translator: {}", e))?;

    // Validate confidence range
    let confidence = output.confidence.clamp(0.0, 1.0);

    // Filter probe IDs to only valid ones
    let needs_probes = filter_valid_probes(output.needs_probes);

    let ticket = TranslatorTicket {
        intent: parse_intent(&output.intent),
        domain: parse_domain(&output.domain),
        entities: output.entities,
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

    let domain = if q.contains("network")
        || q.contains("ip ")
        || q.contains("interface")
        || q.contains("dns")
        || q.contains("port")
        || q.contains("route")
    {
        SpecialistDomain::Network
    } else if q.contains("disk")
        || q.contains("storage")
        || q.contains("space")
        || q.contains("mount")
        || q.contains("partition")
    {
        SpecialistDomain::Storage
    } else if q.contains("security")
        || q.contains("firewall")
        || q.contains("permission")
        || q.contains("ssh")
    {
        SpecialistDomain::Security
    } else if q.contains("package")
        || q.contains("install")
        || q.contains("pacman")
        || q.contains("apt")
    {
        SpecialistDomain::Packages
    } else {
        SpecialistDomain::System
    };

    let intent = if q.contains("install")
        || q.contains("start")
        || q.contains("stop")
        || q.contains("configure")
    {
        QueryIntent::Request
    } else if q.contains("why") || q.contains("debug") || q.contains("fix") {
        QueryIntent::Investigate
    } else {
        QueryIntent::Question
    };

    // Basic probe selection
    let mut needs_probes = Vec::new();
    if q.contains("memory") || q.contains("ram") {
        needs_probes.push("top_memory".to_string());
        needs_probes.push("memory_info".to_string());
    }
    if q.contains("cpu") {
        needs_probes.push("top_cpu".to_string());
        needs_probes.push("cpu_info".to_string());
    }
    if q.contains("disk") || q.contains("space") {
        needs_probes.push("disk_usage".to_string());
    }
    if q.contains("network") || q.contains("ip") {
        needs_probes.push("network_addrs".to_string());
    }
    if q.contains("port") || q.contains("listen") {
        needs_probes.push("listening_ports".to_string());
    }

    TranslatorTicket {
        intent,
        domain,
        entities: Vec::new(),
        needs_probes,
        clarification_question: None,
        confidence: 0.3, // Low confidence for fallback
    }
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
        // Test with typical query
        let input = TranslatorInput::new("what processes are using the most memory", 8, 16.0, true);
        let payload = build_translator_request(&input);

        // Payload must be under 8KB
        assert!(
            payload.len() < MAX_TRANSLATOR_PAYLOAD_SIZE,
            "Translator payload {} bytes exceeds max {} bytes",
            payload.len(),
            MAX_TRANSLATOR_PAYLOAD_SIZE
        );

        // Should be well under 2KB for typical requests
        assert!(
            payload.len() < 2048,
            "Translator payload {} bytes should be under 2KB",
            payload.len()
        );
    }

    #[test]
    fn test_translator_payload_minimal() {
        let input = TranslatorInput::new("show disk space", 4, 8.0, false);
        let payload = build_translator_request(&input);
        // No probe output in payload
        assert!(!payload.contains("USER       PID"));
        assert!(!payload.contains("/dev/sda"));
        assert!(!payload.contains("stdout"));
        // HW summary present
        assert!(payload.contains("CPU cores: 4"));
    }
}
