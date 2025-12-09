//! Service desk architecture with internal roles.
//!
//! Roles (internal, not CLI commands):
//! - Translator: converts user text to structured ticket (LLM-based)
//! - Dispatcher: runs probes based on ticket
//! - Specialist: domain expert generates answer
//! - Supervisor: validates output, assigns reliability score

use anna_shared::reliability::{
    compute_reliability, query_requires_evidence, ReliabilityExplanation, ReliabilityInput,
    ReliabilityOutput,
};
use anna_shared::rpc::{
    Capabilities, EvidenceBlock, HardwareSummary, ProbeResult, ReliabilitySignals, RuntimeContext,
    ServiceDeskResult, SpecialistDomain, TranslatorTicket,
};
use anna_shared::trace::{FallbackUsed, SpecialistOutcome};
use anna_shared::transcript::Transcript;
use anna_shared::VERSION;
use std::collections::HashMap;
use tracing::info;

use crate::scoring;

// Re-export prompt building for backwards compatibility
pub use crate::prompts::{build_specialist_prompt, PromptResult};

// Re-export response builders from dedicated module (v0.0.155)
pub use crate::response_builders::{
    create_no_data_response, create_no_evidence_response, create_timeout_response,
};

// Re-export clarification builders from dedicated module (v0.0.156)
pub use crate::clarification_builders::{
    create_clarification_response, create_clarification_response_grounded,
    create_clarification_with_options,
};

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
            // v0.0.260: Add OS info
            os_name: hardware.os_name.clone(),
            kernel: hardware.kernel.clone(),
            distro: hardware.distro.clone(),
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

/// Build final ServiceDeskResult with explicit flags for scoring
pub fn build_result_with_flags(
    request_id: String,
    answer: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
    translator_timed_out: bool,
    used_deterministic: bool,
    parsed_data_count: usize,
    prompt_truncated: bool,
    fallback_ctx: FallbackContext,
) -> ServiceDeskResult {
    // COST: Check if transcript was capped
    let transcript_capped = transcript.was_capped();
    if transcript_capped {
        if let Some(diag) = transcript.diagnostic() {
            info!("COST: {}", diag.format());
        }
    }

    let (signals, output, input) = calculate_reliability_v2(
        &ticket,
        &probe_results,
        &answer,
        query,
        translator_timed_out,
        used_deterministic,
        parsed_data_count,
        prompt_truncated,
        transcript_capped,
        &fallback_ctx,
    );

    let last_error = if used_deterministic && parsed_data_count == 0 {
        Some("parser produced empty result".to_string())
    } else {
        None
    };
    let evidence = build_evidence(ticket, probe_results, last_error);

    // TRUST: Build explanation if score < 80
    let diagnostics = transcript.diagnostic().into_iter().collect();
    let reliability_explanation = ReliabilityExplanation::build(&output, &input, diagnostics);

    // Log with new reason codes
    let reason_str: Vec<&str> = output.reasons.iter().map(|r| r.explanation()).collect();
    info!(
        "Supervisor: reliability={} reasons={:?} (confident={}, coverage={}, grounded={}, no_invention={})",
        output.score,
        reason_str,
        signals.translator_confident,
        signals.probe_coverage,
        signals.answer_grounded,
        signals.no_invention,
    );

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        reliability_score: output.score,
        reliability_signals: signals,
        reliability_explanation,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None, // Populated by caller
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// Fallback context for TRUST+ explanations and v0.0.24 trace-based scoring
#[derive(Default)]
pub struct FallbackContext {
    pub used_deterministic_fallback: bool,
    pub fallback_route_class: String,
    pub evidence_kinds: Vec<String>,
    /// Trace-based fields (v0.0.24) - source of truth for fallback guardrail
    pub specialist_outcome: Option<SpecialistOutcome>,
    pub fallback_used: Option<FallbackUsed>,
    /// v0.45.x: Evidence requirement from route capability (probe spine enforcement)
    pub evidence_required: Option<bool>,
}


/// Calculate reliability using the new graduated scoring model.
/// Returns (signals for backward compat, output, input) for explanation building.
fn calculate_reliability_v2(
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    answer: &str,
    query: &str,
    translator_timed_out: bool,
    used_deterministic: bool,
    parsed_data_count: usize,
    prompt_truncated: bool,
    transcript_capped: bool,
    fallback_ctx: &FallbackContext,
) -> (ReliabilitySignals, ReliabilityOutput, ReliabilityInput) {
    // Count probe outcomes
    let planned_probes = ticket.needs_probes.len();
    let succeeded_probes = probe_results.iter().filter(|p| p.exit_code == 0).count();
    let failed_probes = probe_results
        .iter()
        .filter(|p| p.exit_code != 0 && !p.stderr.to_lowercase().contains("timeout"))
        .count();
    let timed_out_probes = probe_results
        .iter()
        .filter(|p| p.stderr.to_lowercase().contains("timeout"))
        .count();

    // Evidence requirement: prefer route capability (v0.45.x probe spine), fall back to heuristic
    let evidence_required = fallback_ctx
        .evidence_required
        .unwrap_or_else(|| query_requires_evidence(query));

    // v0.45.4: Parse probes and compute grounding via claim extraction
    use anna_shared::claims::extract_claims;
    use anna_shared::grounding::{
        compute_grounding, is_answer_grounded as grounding_check, ParsedEvidence,
    };
    use anna_shared::parsers::parse_probe_result;

    // Parse probe results into structured data
    let parsed_probes: Vec<_> = probe_results.iter().map(parse_probe_result).collect();

    // Build evidence from parsed probes
    let evidence = ParsedEvidence::from_probes(&parsed_probes);

    // Extract claims from answer and compute grounding
    let claims = extract_claims(answer);
    let grounding_report = compute_grounding(&claims, &evidence);

    // Determine answer quality using grounding (v0.45.4)
    let answer_grounded = if used_deterministic {
        parsed_data_count > 0
    } else if grounding_report.total_claims > 0 {
        // Use claim-based grounding when claims were extracted
        grounding_check(&grounding_report)
    } else {
        // Fallback to heuristic when no claims extracted
        scoring::check_answer_grounded(answer, probe_results)
    };

    // Use GUARD-based invention check when we have evidence
    let no_invention = if used_deterministic {
        parsed_data_count > 0
    } else {
        scoring::check_no_invention_guard(answer, &evidence, evidence_required)
    };

    // Build input for new scoring model (v0.45.4: full claim integration)
    let input = ReliabilityInput {
        planned_probes,
        succeeded_probes,
        failed_probes,
        timed_out_probes,
        translator_confidence: ticket.confidence,
        translator_used: !used_deterministic && !translator_timed_out,
        answer_grounded,
        no_invention,
        grounding_ratio: grounding_report.grounding_ratio,
        total_claims: grounding_report.total_claims,
        evidence_required,
        used_deterministic,
        parsed_data_count,
        prompt_truncated,
        transcript_capped,
        // METER phase: budget enforcement (integrated via rpc_handler)
        budget_exceeded: false,
        exceeded_stage: None,
        stage_budget_ms: 0,
        stage_elapsed_ms: 0,
        // TRUST+ phase: fallback context
        used_deterministic_fallback: fallback_ctx.used_deterministic_fallback,
        fallback_route_class: fallback_ctx.fallback_route_class.clone(),
        evidence_kinds: fallback_ctx.evidence_kinds.clone(),
        // Trace context (v0.0.24) - source of truth for fallback guardrail
        specialist_outcome: fallback_ctx.specialist_outcome,
        fallback_used: fallback_ctx.fallback_used.clone(),
    };

    // Compute using new model
    let output = compute_reliability(&input);

    // Build backward-compatible signals from output
    let signals = ReliabilitySignals {
        translator_confident: !translator_timed_out && ticket.confidence >= 0.7,
        probe_coverage: output.probe_coverage_ratio >= 1.0,
        answer_grounded,
        no_invention,
        clarification_not_needed: !answer.is_empty()
            && (!used_deterministic || parsed_data_count > 0),
    };

    (signals, output, input)
}

// Unit tests moved to tests/service_desk_tests.rs
