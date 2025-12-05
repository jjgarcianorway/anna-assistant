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
use anna_shared::transcript::Transcript;
use anna_shared::VERSION;
use std::collections::HashMap;
use tracing::info;

use crate::scoring;

// Re-export build_specialist_prompt for backwards compatibility
pub use crate::prompts::build_specialist_prompt;

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

    if ticket.domain == SpecialistDomain::System {
        fields.push("cpu_model".to_string());
        fields.push("cpu_cores".to_string());
        fields.push("ram_gb".to_string());
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

/// Maximum reliability score for clarification responses
pub const CLARIFICATION_MAX_RELIABILITY: u8 = 40;

/// Create a clarification response
pub fn create_clarification_response(
    request_id: String,
    ticket: TranslatorTicket,
    question: &str,
    transcript: Transcript,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };
    // Ensure reliability is capped for clarification
    let score = signals.score().min(CLARIFICATION_MAX_RELIABILITY);

    let evidence = EvidenceBlock {
        hardware_fields: vec![],
        probes_executed: vec![],
        translator_ticket: ticket,
        last_error: None,
    };

    ServiceDeskResult {
        request_id,
        answer: String::new(),
        reliability_score: score,
        reliability_signals: signals,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(question.to_string()),
        transcript,
    }
}

/// Create a timeout error response
pub fn create_timeout_response(
    request_id: String,
    stage: &str,
    ticket: Option<TranslatorTicket>,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
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
        domain,
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
        request_id,
        answer: String::new(),
        reliability_score: signals.score().min(20), // Max 20 for timeout
        reliability_signals: signals,
        domain,
        evidence,
        needs_clarification: true,
        clarification_question: Some(format!(
            "The {} stage timed out. Please try again or simplify your request.",
            stage
        )),
        transcript,
    }
}

/// Create a response when no data is available to answer
pub fn create_no_data_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: !probe_results.is_empty(),
        answer_grounded: false,
        no_invention: true,
        clarification_not_needed: false,
    };

    let evidence = build_evidence(
        ticket,
        probe_results,
        Some("No deterministic answer available".to_string()),
    );

    ServiceDeskResult {
        request_id,
        answer: String::new(),
        reliability_score: signals.score(),
        reliability_signals: signals,
        domain,
        evidence,
        needs_clarification: true,
        clarification_question: Some(
            "I couldn't generate an answer from the available data. Could you rephrase your question?".to_string()
        ),
        transcript,
    }
}

/// Build final ServiceDeskResult with explicit flags for scoring
pub fn build_result_with_flags(
    request_id: String,
    answer: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
    translator_timed_out: bool,
    used_deterministic: bool,
    parsed_data_count: usize,
) -> ServiceDeskResult {
    let signals = calculate_signals_with_flags(
        &ticket,
        &probe_results,
        &answer,
        translator_timed_out,
        used_deterministic,
        parsed_data_count,
    );
    let score = signals.score();
    let last_error = if used_deterministic && parsed_data_count == 0 {
        Some("parser produced empty result".to_string())
    } else {
        None
    };
    let evidence = build_evidence(ticket, probe_results, last_error);

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
        request_id,
        answer,
        reliability_score: score,
        reliability_signals: signals,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        transcript,
    }
}

/// Calculate signals with explicit timeout/deterministic flags
fn calculate_signals_with_flags(
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    answer: &str,
    translator_timed_out: bool,
    used_deterministic: bool,
    parsed_data_count: usize,
) -> ReliabilitySignals {
    // Translator confident only if it didn't timeout and has high confidence
    let translator_confident = !translator_timed_out && ticket.confidence >= 0.7;

    // Probe coverage: all requested probes succeeded
    let probe_coverage = if ticket.needs_probes.is_empty() {
        true
    } else {
        let successful = probe_results.iter().filter(|p| p.exit_code == 0).count();
        successful >= ticket.needs_probes.len()
    };

    // Deterministic answers are grounded only if they actually parsed data
    let answer_grounded = if used_deterministic {
        parsed_data_count > 0
    } else {
        scoring::check_answer_grounded(answer, probe_results)
    };

    // Deterministic answers don't invent (unless empty result)
    let no_invention = if used_deterministic {
        parsed_data_count > 0
    } else {
        scoring::check_no_invention(answer)
    };

    // Clarification not needed if we have an answer with actual content
    let clarification_not_needed = !answer.is_empty() && (!used_deterministic || parsed_data_count > 0);

    ReliabilitySignals {
        translator_confident,
        probe_coverage,
        answer_grounded,
        no_invention,
        clarification_not_needed,
    }
}

// Unit tests moved to tests/service_desk_tests.rs
