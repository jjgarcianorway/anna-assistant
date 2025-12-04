//! Service desk architecture with internal roles.
//!
//! Roles (internal, not CLI commands):
//! - Translator: converts user text to structured ticket (LLM-based)
//! - Dispatcher: runs probes based on ticket
//! - Specialist: domain expert generates answer
//! - Supervisor: validates output, assigns reliability score

use anna_shared::rpc::{
    Capabilities, EvidenceBlock, HardwareSummary, ProbeResult, ReliabilitySignals, RuntimeContext,
    ServiceDeskResult, SpecialistDomain, TranslatorTicket,
};
use anna_shared::VERSION;
use std::collections::HashMap;
use tracing::info;

use crate::scoring;

/// Allowlist of read-only probes that specialists can request
pub const ALLOWED_PROBES: &[&str] = &[
    "ps aux --sort=-%mem",
    "ps aux --sort=-%cpu",
    "lscpu",
    "free -h",
    "df -h",
    "lsblk",
    "ip addr show",
    "ip route",
    "ss -tulpn",
    "systemctl --failed",
    "journalctl -p warning..alert -n 200 --no-pager",
];

/// Check if a probe is in the allowlist
#[allow(dead_code)]
pub fn is_probe_allowed(probe: &str) -> bool {
    ALLOWED_PROBES.iter().any(|p| probe.starts_with(p))
}

/// Build runtime context for LLM
pub fn build_context(
    hardware: &anna_shared::status::HardwareInfo,
    probe_results: &[ProbeResult],
) -> RuntimeContext {
    // Convert structured probe results to HashMap for context
    let probes: HashMap<String, String> = probe_results
        .iter()
        .filter(|p| p.exit_code == 0)
        .map(|p| (p.command.clone(), p.stdout.clone()))
        .collect();

    RuntimeContext {
        version: VERSION.to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: hardware.cpu_model.clone(),
            cpu_cores: hardware.cpu_cores,
            ram_gb: hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            gpu: hardware.gpu.as_ref().map(|g| g.model.clone()),
            gpu_vram_gb: hardware
                .gpu
                .as_ref()
                .map(|g| g.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
        },
        probes,
    }
}

/// Determine which hardware fields are relevant to the query
pub fn get_relevant_hardware_fields(ticket: &TranslatorTicket) -> Vec<String> {
    let mut fields = Vec::new();

    // Always include version
    fields.push("version".to_string());

    match ticket.domain {
        SpecialistDomain::System => {
            fields.push("cpu_model".to_string());
            fields.push("cpu_cores".to_string());
            fields.push("ram_gb".to_string());
        }
        SpecialistDomain::Network => {
            // Network doesn't use hardware fields directly
        }
        SpecialistDomain::Storage => {
            // Storage uses probes, not hardware snapshot
        }
        SpecialistDomain::Security => {
            // Security uses probes
        }
        SpecialistDomain::Packages => {
            // Packages don't need hardware
        }
    }

    // GPU if relevant
    if ticket
        .entities
        .iter()
        .any(|e| e.to_lowercase().contains("gpu"))
    {
        fields.push("gpu".to_string());
        fields.push("gpu_vram_gb".to_string());
    }

    fields
}

/// Build evidence block from ticket and probe results
pub fn build_evidence(
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    last_error: Option<String>,
) -> EvidenceBlock {
    let hardware_fields = get_relevant_hardware_fields(&ticket);

    EvidenceBlock {
        hardware_fields,
        probes_executed: probe_results,
        translator_ticket: ticket,
        last_error,
    }
}

/// Build grounded system prompt with runtime context for specialist
pub fn build_specialist_prompt(
    domain: SpecialistDomain,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> String {
    let specialist_intro = match domain {
        SpecialistDomain::System => {
            "You are the System Specialist, expert in CPU, memory, processes, and services."
        }
        SpecialistDomain::Network => {
            "You are the Network Specialist, expert in interfaces, routing, DNS, and connectivity."
        }
        SpecialistDomain::Storage => {
            "You are the Storage Specialist, expert in disks, partitions, mounts, and filesystems."
        }
        SpecialistDomain::Security => {
            "You are the Security Specialist, expert in permissions, firewalls, and audit logs."
        }
        SpecialistDomain::Packages => {
            "You are the Package Specialist, expert in package managers and software installation."
        }
    };

    let mut prompt = format!(
        r#"You are Anna, a local AI assistant running on this Linux machine.
{specialist_intro}

=== RUNTIME CONTEXT (AUTHORITATIVE - DO NOT CONTRADICT) ===
Version: {}
Daemon: running

Hardware (from system probe):
  - CPU: {} ({} cores)
  - RAM: {:.1} GB"#,
        context.version,
        context.hardware.cpu_model,
        context.hardware.cpu_cores,
        context.hardware.ram_gb,
    );

    if let Some(gpu) = &context.hardware.gpu {
        if let Some(vram) = context.hardware.gpu_vram_gb {
            prompt.push_str(&format!("\n  - GPU: {} ({:.1} GB VRAM)", gpu, vram));
        } else {
            prompt.push_str(&format!("\n  - GPU: {}", gpu));
        }
    } else {
        prompt.push_str("\n  - GPU: none");
    }

    // Add probe results if any
    if !probe_results.is_empty() {
        prompt.push_str("\n\n=== PROBE RESULTS ===");
        for probe in probe_results {
            if probe.exit_code == 0 {
                prompt.push_str(&format!("\n[{}]\n{}", probe.command, probe.stdout));
            } else {
                prompt.push_str(&format!(
                    "\n[{}] FAILED (exit {}): {}",
                    probe.command, probe.exit_code, probe.stderr
                ));
            }
        }
    }

    prompt.push_str(
        r#"

=== GROUNDING RULES (MANDATORY) ===
1. NEVER invent or guess information not in the runtime context above.
2. For hardware questions: Answer DIRECTLY from the hardware section above.
3. For process/memory/disk questions: Use the probe results above if available.
4. If data is missing: Say exactly what data is missing.
5. NEVER suggest manual commands when you have the data above.
6. Use the exact version shown above when discussing Anna's version.
7. Reference the specific data you're using in your answer.

=== END CONTEXT ===

Answer using ONLY the data provided above. Be concise and direct."#,
    );

    prompt
}

/// Calculate reliability signals and score
pub fn calculate_reliability(
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    answer: &str,
) -> (ReliabilitySignals, u8) {
    let signals = scoring::calculate_signals(ticket, probe_results, answer);
    let score = signals.score();
    (signals, score)
}

/// Create a clarification response
pub fn create_clarification_response(
    ticket: TranslatorTicket,
    question: &str,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };

    let evidence = EvidenceBlock {
        hardware_fields: vec![],
        probes_executed: vec![],
        translator_ticket: ticket,
        last_error: None,
    };

    ServiceDeskResult {
        answer: String::new(),
        reliability_score: signals.score(),
        reliability_signals: signals,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(question.to_string()),
    }
}

/// Create a timeout error response
pub fn create_timeout_response(
    stage: &str,
    ticket: Option<TranslatorTicket>,
    probe_results: Vec<ProbeResult>,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };

    let default_ticket = ticket.unwrap_or_else(|| TranslatorTicket {
        intent: anna_shared::rpc::QueryIntent::Question,
        domain: SpecialistDomain::System,
        entities: vec![],
        needs_probes: vec![],
        clarification_question: None,
        confidence: 0.0,
    });

    let evidence = build_evidence(
        default_ticket,
        probe_results,
        Some(format!("timeout at {}", stage)),
    );

    ServiceDeskResult {
        answer: String::new(),
        reliability_score: signals.score().min(20), // Max 20 for timeout
        reliability_signals: signals,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(format!(
            "The {} stage timed out. Please try again or simplify your request.",
            stage
        )),
    }
}

/// Build final ServiceDeskResult
pub fn build_result(
    answer: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
) -> ServiceDeskResult {
    let (signals, score) = calculate_reliability(&ticket, &probe_results, &answer);
    let domain = ticket.domain;
    let evidence = build_evidence(ticket, probe_results, None);

    info!(
        "Supervisor: reliability={} (confident={}, coverage={}, grounded={}, no_invention={}, no_clarify={})",
        score,
        signals.translator_confident,
        signals.probe_coverage,
        signals.answer_grounded,
        signals.no_invention,
        signals.clarification_not_needed
    );

    ServiceDeskResult {
        answer,
        reliability_score: score,
        reliability_signals: signals,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::rpc::QueryIntent;

    fn make_ticket() -> TranslatorTicket {
        TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec!["top_memory".to_string()],
            clarification_question: None,
            confidence: 0.8,
        }
    }

    #[test]
    fn test_is_probe_allowed() {
        assert!(is_probe_allowed("ps aux --sort=-%mem"));
        assert!(is_probe_allowed("df -h"));
        assert!(!is_probe_allowed("rm -rf /"));
    }

    #[test]
    fn test_get_relevant_hardware_fields() {
        let ticket = make_ticket();
        let fields = get_relevant_hardware_fields(&ticket);
        assert!(fields.contains(&"cpu_model".to_string()));
        assert!(fields.contains(&"ram_gb".to_string()));
    }

    #[test]
    fn test_build_evidence() {
        let ticket = make_ticket();
        let probes = vec![ProbeResult {
            command: "ps aux --sort=-%mem".to_string(),
            exit_code: 0,
            stdout: "output".to_string(),
            stderr: String::new(),
            timing_ms: 100,
        }];

        let evidence = build_evidence(ticket, probes, None);
        assert_eq!(evidence.probes_executed.len(), 1);
        assert!(evidence.hardware_fields.contains(&"cpu_model".to_string()));
        assert!(evidence.last_error.is_none());
    }

    #[test]
    fn test_build_evidence_with_error() {
        let ticket = make_ticket();
        let evidence = build_evidence(ticket, vec![], Some("timeout at translator".to_string()));
        assert!(evidence.last_error.is_some());
        assert!(evidence.last_error.unwrap().contains("timeout"));
    }

    #[test]
    fn test_create_timeout_response() {
        let result = create_timeout_response("translator", None, vec![]);
        assert!(result.needs_clarification);
        assert!(result.evidence.last_error.is_some());
        assert!(result.reliability_score <= 20);
    }
}
