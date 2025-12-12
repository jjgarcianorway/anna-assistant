//! Specialist prompt contract (v0.0.425).
//!
//! Strict prompts that enforce JSON-only responses.
//! No tutorials, no explanations - just structured data.

use super::{ResponseStatus, RiskLevel, Severity};

/// System prompt for all specialists.
pub const SPECIALIST_SYSTEM_PROMPT: &str = r#"You are a Linux system specialist. You MUST respond with ONLY valid JSON.

## OUTPUT FORMAT - MANDATORY
Your entire response must be a single JSON object. No markdown, no explanations, no prose.

```
{
  "ticket_id": "<ticket ID from request>",
  "specialist": {
    "name": "<your specialist name>",
    "role": "<your role>",
    "department": "<desktop|server|network|security>"
  },
  "status": "<success|partial|no_data|unsupported|error>",
  "summary": "<one technical sentence>",
  "confidence": <0.0-1.0>,
  "severity": "<info|warning|critical>",
  "findings": [
    {"key": "<metric_name>", "value": "<measured_value>", "evidence_refs": ["probe:<name>"]}
  ],
  "analysis": ["<bullet 1>", "<bullet 2>"],
  "recommendations": [
    {"id": "rec-1", "title": "<short>", "description": "<details>", "risk_level": "low|medium|high"}
  ],
  "actions": [
    {"id": "act-1", "title": "<short>", "command": "<shell cmd>", "run_as": "user|root", "risk_level": "low|medium|high"}
  ],
  "knowledge_citations": [
    {"id": "<citation>", "source": "<man|help|wiki|doc>", "topic": "<topic>", "relevance": "low|medium|high"}
  ],
  "probes_used": [
    {"id": "probe:<name>", "status": "ok|empty|failed|timeout", "description": "<what it checked>"}
  ]
}
```

## STATUS SEMANTICS - CHOOSE CAREFULLY
- `success`: Complete answer with high confidence based on probe data
- `partial`: Some findings but important data missing or inconclusive
- `no_data`: Probes returned nothing useful for this specific question
- `unsupported`: Question is outside your specialist domain
- `error`: Something went wrong (add error.message and error.kind)

## RULES - MUST FOLLOW
1. ONLY output JSON. No explanations before or after.
2. Every finding MUST have evidence_refs pointing to probes or citations
3. Never invent data. If probes are empty, status must be "no_data"
4. Keep summary to ONE sentence
5. Analysis bullets should be 1-4 short items
6. Commands in actions must be safe and specific
7. Confidence must reflect actual evidence quality

## WHAT NOT TO DO
- No "Here's my analysis..."
- No markdown formatting
- No tutorials or explanations
- No generic advice without evidence
- No hallucinated data
"#;

/// Generate a specialist prompt for a specific query.
pub fn build_specialist_prompt(
    ticket_id: &str,
    question: &str,
    probe_data: &[(String, String)],
    knowledge_snippets: &[String],
    specialist_domain: &str,
) -> String {
    let mut prompt = String::with_capacity(4096);

    // Ticket context
    prompt.push_str(&format!("TICKET: {}\n", ticket_id));
    prompt.push_str(&format!("DOMAIN: {}\n", specialist_domain));
    prompt.push_str(&format!("QUESTION: {}\n\n", question));

    // Probe data section
    if !probe_data.is_empty() {
        prompt.push_str("PROBE DATA:\n");
        for (probe_id, output) in probe_data {
            prompt.push_str(&format!(
                "--- {} ---\n{}\n\n",
                probe_id,
                truncate_probe(output)
            ));
        }
    } else {
        prompt.push_str("PROBE DATA: (none available)\n\n");
    }

    // Knowledge section
    if !knowledge_snippets.is_empty() {
        prompt.push_str("KNOWLEDGE CITATIONS:\n");
        for snippet in knowledge_snippets {
            prompt.push_str(&format!("{}\n\n", snippet));
        }
    }

    // Final instruction
    prompt.push_str("Respond with ONLY the JSON object. No other text.");

    prompt
}

/// Truncate probe output to reasonable size.
fn truncate_probe(output: &str) -> &str {
    const MAX_PROBE_CHARS: usize = 2000;
    if output.len() <= MAX_PROBE_CHARS {
        output
    } else {
        &output[..MAX_PROBE_CHARS]
    }
}

/// Domain-specific prompt additions.
pub struct DomainPrompt {
    pub domain: String,
    pub focus_areas: Vec<String>,
    pub common_probes: Vec<String>,
}

impl DomainPrompt {
    /// Desktop specialist prompt additions.
    pub fn desktop() -> Self {
        Self {
            domain: "desktop".to_string(),
            focus_areas: vec![
                "Memory and swap usage".to_string(),
                "Disk space and I/O".to_string(),
                "Display and GPU status".to_string(),
                "Audio/PipeWire/PulseAudio".to_string(),
                "Desktop environment issues".to_string(),
            ],
            common_probes: vec![
                "probe:free".to_string(),
                "probe:df".to_string(),
                "probe:top".to_string(),
                "probe:systemctl_user".to_string(),
            ],
        }
    }

    /// Server specialist prompt additions.
    pub fn server() -> Self {
        Self {
            domain: "server".to_string(),
            focus_areas: vec![
                "Service health and status".to_string(),
                "Resource utilization".to_string(),
                "Log analysis".to_string(),
                "Container status".to_string(),
            ],
            common_probes: vec![
                "probe:systemctl".to_string(),
                "probe:journalctl".to_string(),
                "probe:docker_ps".to_string(),
                "probe:ss".to_string(),
            ],
        }
    }

    /// Network specialist prompt additions.
    pub fn network() -> Self {
        Self {
            domain: "network".to_string(),
            focus_areas: vec![
                "Network interfaces".to_string(),
                "DNS resolution".to_string(),
                "Firewall rules".to_string(),
                "Connection status".to_string(),
            ],
            common_probes: vec![
                "probe:ip_addr".to_string(),
                "probe:ss".to_string(),
                "probe:resolvectl".to_string(),
                "probe:ping".to_string(),
            ],
        }
    }

    /// Security specialist prompt additions.
    pub fn security() -> Self {
        Self {
            domain: "security".to_string(),
            focus_areas: vec![
                "Authentication logs".to_string(),
                "Failed login attempts".to_string(),
                "Firewall status".to_string(),
                "Package integrity".to_string(),
            ],
            common_probes: vec![
                "probe:lastlog".to_string(),
                "probe:journalctl_auth".to_string(),
                "probe:iptables".to_string(),
                "probe:pacman_check".to_string(),
            ],
        }
    }

    /// Get domain prompt by name.
    pub fn for_domain(domain: &str) -> Self {
        match domain {
            "desktop" => Self::desktop(),
            "server" => Self::server(),
            "network" => Self::network(),
            "security" => Self::security(),
            _ => Self::desktop(), // Default to desktop
        }
    }

    /// Add domain context to prompt.
    pub fn augment_prompt(&self, base_prompt: &str) -> String {
        format!(
            "{}\n\nDOMAIN FOCUS: {}\nKEY AREAS: {}\nCOMMON PROBES: {}",
            base_prompt,
            self.domain,
            self.focus_areas.join(", "),
            self.common_probes.join(", ")
        )
    }
}

/// Example response for the specialist to follow.
pub fn example_success_response() -> &'static str {
    r#"{
  "ticket_id": "DSK-001",
  "specialist": {"name": "Sofia", "role": "System Admin", "department": "desktop"},
  "status": "success",
  "summary": "Memory usage is healthy at 33% with 17GB available",
  "confidence": 0.95,
  "severity": "info",
  "findings": [
    {"key": "mem_total_gb", "value": "25.6", "evidence_refs": ["probe:free"]},
    {"key": "mem_used_gb", "value": "8.4", "evidence_refs": ["probe:free"]},
    {"key": "mem_available_gb", "value": "17.0", "evidence_refs": ["probe:free"]},
    {"key": "swap_used_mb", "value": "0", "evidence_refs": ["probe:free"]}
  ],
  "analysis": [
    "Memory utilization at 33% is well within healthy range",
    "No swap usage indicates sufficient RAM",
    "17GB available provides good headroom for new applications"
  ],
  "recommendations": [],
  "actions": [],
  "knowledge_citations": [],
  "probes_used": [
    {"id": "probe:free", "status": "ok", "description": "Memory usage statistics"}
  ]
}"#
}

/// Example no-data response.
pub fn example_no_data_response() -> &'static str {
    r#"{
  "ticket_id": "DSK-002",
  "specialist": {"name": "Sofia", "role": "System Admin", "department": "desktop"},
  "status": "no_data",
  "summary": "No GPU information available - probe returned empty",
  "confidence": 0.1,
  "severity": "info",
  "findings": [],
  "analysis": [
    "lspci probe did not return GPU information",
    "This may indicate no discrete GPU or driver issues"
  ],
  "recommendations": [
    {"id": "rec-1", "title": "Check drivers", "description": "Verify GPU drivers are installed", "risk_level": "low"}
  ],
  "actions": [
    {"id": "act-1", "title": "List PCI devices", "command": "lspci -v | grep -i vga", "run_as": "user", "risk_level": "low"}
  ],
  "knowledge_citations": [],
  "probes_used": [
    {"id": "probe:lspci", "status": "empty", "description": "PCI device listing"}
  ]
}"#
}

/// Confidence guidelines for specialists.
pub fn confidence_guidelines() -> &'static str {
    r#"CONFIDENCE SCORING:
- 0.9-1.0: Direct probe data answers the question completely
- 0.7-0.9: Strong evidence with minor gaps
- 0.5-0.7: Partial evidence, some inference needed
- 0.3-0.5: Limited evidence, significant inference
- 0.0-0.3: Minimal evidence, mostly educated guess"#
}

/// Map severity to risk assessment.
pub fn severity_for_finding(key: &str, value: &str) -> Severity {
    // Memory thresholds
    if key.contains("mem_available") || key.contains("mem_free") {
        if let Ok(mb) = value.parse::<u64>() {
            return if mb < 500 {
                Severity::Critical
            } else if mb < 2000 {
                Severity::Warning
            } else {
                Severity::Info
            };
        }
    }

    // Disk thresholds
    if key.contains("disk_free") || key.contains("disk_available") {
        if let Ok(gb) = value.parse::<f64>() {
            return if gb < 1.0 {
                Severity::Critical
            } else if gb < 5.0 {
                Severity::Warning
            } else {
                Severity::Info
            };
        }
    }

    // Default
    Severity::Info
}

/// Map action risk based on command patterns.
pub fn risk_for_command(command: &str) -> RiskLevel {
    let cmd_lower = command.to_lowercase();

    // High risk patterns
    if cmd_lower.contains("rm -rf")
        || cmd_lower.contains("dd if=")
        || cmd_lower.contains("mkfs")
        || cmd_lower.contains("> /dev/")
    {
        return RiskLevel::High;
    }

    // Medium risk patterns
    if cmd_lower.contains("sudo")
        || cmd_lower.contains("systemctl")
        || cmd_lower.contains("pacman -R")
        || cmd_lower.contains("kill")
    {
        return RiskLevel::Medium;
    }

    // Low risk (default)
    RiskLevel::Low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let prompt = build_specialist_prompt(
            "DSK-001",
            "How much memory is available?",
            &[(
                "probe:free".to_string(),
                "Mem: 25600 8400 17000".to_string(),
            )],
            &[],
            "desktop",
        );

        assert!(prompt.contains("DSK-001"));
        assert!(prompt.contains("memory"));
        assert!(prompt.contains("probe:free"));
    }

    #[test]
    fn test_domain_prompts() {
        let desktop = DomainPrompt::desktop();
        assert_eq!(desktop.domain, "desktop");
        assert!(!desktop.focus_areas.is_empty());

        let server = DomainPrompt::server();
        assert_eq!(server.domain, "server");
    }

    #[test]
    fn test_severity_for_finding() {
        assert_eq!(
            severity_for_finding("mem_available_mb", "100"),
            Severity::Critical
        );
        assert_eq!(
            severity_for_finding("mem_available_mb", "1000"),
            Severity::Warning
        );
        assert_eq!(
            severity_for_finding("mem_available_mb", "8000"),
            Severity::Info
        );
    }

    #[test]
    fn test_risk_for_command() {
        assert_eq!(risk_for_command("rm -rf /tmp/*"), RiskLevel::High);
        assert_eq!(
            risk_for_command("sudo systemctl restart foo"),
            RiskLevel::Medium
        );
        assert_eq!(risk_for_command("ls -la"), RiskLevel::Low);
    }
}
