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

/// Build the translator system prompt
fn build_translator_prompt() -> String {
    format!(
        r#"You are a query translator for a Linux system assistant. Your job is to analyze user queries and output a structured JSON ticket.

STRICT OUTPUT FORMAT - You must respond with ONLY valid JSON:
{{
  "intent": "<question|request|investigate>",
  "domain": "<system|network|storage|security|packages>",
  "entities": ["list", "of", "extracted", "entities"],
  "needs_probes": ["probe_ids", "from", "allowlist"],
  "clarification_question": null or "question string if query is ambiguous",
  "confidence": 0.0 to 1.0
}}

DOMAIN RULES:
- system: CPU, memory, processes, services, systemd, logs
- network: IP addresses, interfaces, routes, DNS, ports, connections
- storage: disks, partitions, filesystems, mounts, space
- security: firewalls, permissions, SELinux, AppArmor, SSH, audit
- packages: apt, dnf, pacman, yum, install, update, upgrade

INTENT RULES:
- question: user wants information (what, which, how much, show me)
- request: user wants action (install, start, stop, configure)
- investigate: user wants diagnosis (why, debug, troubleshoot, fix)

AVAILABLE PROBE IDS (only select from this list):
{}

PROBE SELECTION RULES:
- For memory questions: top_memory, memory_info
- For CPU questions: top_cpu, cpu_info
- For disk/storage: disk_usage, block_devices
- For network: network_addrs, network_routes, listening_ports
- For services/errors: failed_services, system_logs
- Select ONLY probes that will help answer the query

CLARIFICATION RULES:
- Set clarification_question if query is too vague (e.g., "help", "fix it")
- Set confidence < 0.5 if unsure about domain or intent
- Short queries (1-2 words) without clear intent need clarification

RESPOND WITH JSON ONLY. NO OTHER TEXT."#,
        PROBE_IDS.join(", ")
    )
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

/// Translate user query to structured ticket using LLM
pub async fn translate(model: &str, query: &str) -> Result<TranslatorTicket, String> {
    let system_prompt = build_translator_prompt();
    let full_prompt = format!("{}\n\nUser query: {}", system_prompt, query);

    info!("Translator: processing query");

    let response = ollama::chat(model, &full_prompt)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;

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
    let trimmed = response.trim();

    // Try direct parse first
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Ok(trimmed.to_string());
    }

    // Try to extract from markdown code block
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start..]
            .find("```\n")
            .or(trimmed[start..].rfind("```"))
        {
            let json_start = start + 7; // skip ```json
            let json_end = start + end;
            if json_start < json_end {
                return Ok(trimmed[json_start..json_end].trim().to_string());
            }
        }
    }

    // Try to extract from plain code block
    if let Some(start) = trimmed.find("```") {
        let after_start = start + 3;
        if let Some(end) = trimmed[after_start..].find("```") {
            let json_part = &trimmed[after_start..after_start + end];
            // Skip language identifier if present
            let json_str = json_part
                .lines()
                .skip_while(|l| !l.trim().starts_with('{'))
                .collect::<Vec<_>>()
                .join("\n");
            if !json_str.is_empty() {
                return Ok(json_str);
            }
        }
    }

    // Try to find JSON object anywhere in response
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if start < end {
                return Ok(trimmed[start..=end].to_string());
            }
        }
    }

    Err("No valid JSON found in translator response".to_string())
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
}
