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

/// Create a deterministic "no evidence" failure response (v0.45.4).
/// Used when evidence_required=true but probe_stats.succeeded==0.
pub fn create_no_evidence_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
    required_evidence: &[String],
) -> ServiceDeskResult {
    // v0.45.4: Deterministic failure format
    let evidence_list = if required_evidence.is_empty() {
        "system data".to_string()
    } else {
        required_evidence.join(", ")
    };
    let answer = format!(
        "I can't answer yet because I didn't collect evidence for: {}. Run: `annactl status` and retry.",
        evidence_list
    );

    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false,
        no_invention: true, // No claims made = no invention
        clarification_not_needed: true, // We gave a clear answer
    };

    let evidence = build_evidence(
        ticket,
        probe_results,
        Some("no probes succeeded".to_string()),
    );

    ServiceDeskResult {
        request_id,
        answer,
        reliability_score: anna_shared::reliability::NO_EVIDENCE_RELIABILITY_CAP,
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence,
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None,
    }
}

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
        reliability_explanation: None, // No explanation for clarification
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(question.to_string()),
        clarification_request: None, // TODO: Build ClarifyRequest from context
        transcript,
        execution_trace: None, // Clarification response - no trace needed
    }
}

/// Create a timeout error response with evidence summary (v0.45.x stabilization).
/// Never asks to rephrase - always provides factual status.
pub fn create_timeout_response(
    request_id: String,
    stage: &str,
    ticket: Option<TranslatorTicket>,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
) -> ServiceDeskResult {
    // v0.45.x: Build evidence summary from available probe data
    let answer = build_timeout_evidence_summary(stage, &probe_results);

    let has_evidence = !probe_results.is_empty()
        && probe_results.iter().any(|p| p.exit_code == 0);

    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: has_evidence,
        answer_grounded: has_evidence,
        no_invention: true,
        clarification_not_needed: true, // Never ask to rephrase
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
        answer,
        reliability_score: if has_evidence { 40 } else { 20 }, // Higher if we have evidence
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence,
        needs_clarification: false, // Never ask to rephrase
        clarification_question: None, // v0.45.x: No clarification - we provide status
        clarification_request: None,
        transcript,
        execution_trace: None, // Populated by caller if needed
    }
}

/// Build evidence summary for timeout response (v0.45.x stabilization).
fn build_timeout_evidence_summary(stage: &str, probe_results: &[ProbeResult]) -> String {
    let mut answer = format!("**Timeout at {} stage**\n\n", stage);

    let successful: Vec<_> = probe_results.iter().filter(|p| p.exit_code == 0).collect();
    let failed: Vec<_> = probe_results.iter().filter(|p| p.exit_code != 0).collect();

    if successful.is_empty() && failed.is_empty() {
        answer.push_str("No probes were completed before the timeout.\n\n");
    } else {
        if !successful.is_empty() {
            answer.push_str("**Evidence gathered before timeout:**\n\n");
            for probe in &successful {
                // Extract meaningful output (first 3 lines)
                let output: String = probe.stdout
                    .lines()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("\n");
                if !output.trim().is_empty() {
                    let truncated = if probe.stdout.lines().count() > 3 { "..." } else { "" };
                    answer.push_str(&format!(
                        "- `{}`: {}{}\n",
                        probe.command,
                        output.replace('\n', " | "),
                        truncated
                    ));
                }
            }
            answer.push('\n');
        }

        if !failed.is_empty() {
            answer.push_str(&format!(
                "{} probe{} failed before timeout.\n",
                failed.len(),
                if failed.len() == 1 { "" } else { "s" }
            ));
        }
    }

    answer.push_str("*The request exceeded its time budget. Try a more specific query.*");
    answer
}

/// Create a best-effort response when no deterministic answer is available (v0.0.32).
/// Always answers - never asks to rephrase.
pub fn create_no_data_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    domain: SpecialistDomain,
) -> ServiceDeskResult {
    // Build a best-effort answer from available probe data
    let answer = build_best_effort_answer(&probe_results, domain);

    let signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: !probe_results.is_empty(),
        answer_grounded: !probe_results.is_empty(),
        no_invention: true,
        clarification_not_needed: true, // We always provide an answer now
    };

    let evidence = build_evidence(
        ticket,
        probe_results,
        Some("Best-effort answer from available data".to_string()),
    );

    ServiceDeskResult {
        request_id,
        answer,
        reliability_score: signals.score(),
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence,
        needs_clarification: false, // Never ask for rephrase
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None,
    }
}

/// Build a best-effort answer from available probe results (v0.0.32).
/// Summarizes what data was gathered even if it's incomplete.
fn build_best_effort_answer(probe_results: &[ProbeResult], domain: SpecialistDomain) -> String {
    if probe_results.is_empty() {
        return format!(
            "I was unable to gather data for this {} query. \
             The system probes didn't return results. \
             Please ensure the relevant services are running and try again.",
            domain
        );
    }

    let successful: Vec<_> = probe_results.iter().filter(|p| p.exit_code == 0).collect();
    let failed: Vec<_> = probe_results.iter().filter(|p| p.exit_code != 0).collect();

    let mut answer = String::new();

    if !successful.is_empty() {
        answer.push_str("Based on the available system data:\n\n");

        for probe in &successful {
            // Extract meaningful output (first 3 lines or 200 chars)
            let output = probe.stdout.lines().take(3).collect::<Vec<_>>().join("\n");
            let output = if output.len() > 200 {
                format!("{}...", &output[..200])
            } else {
                output
            };

            if !output.trim().is_empty() {
                answer.push_str(&format!("**{}**:\n```\n{}\n```\n\n", probe.command, output));
            }
        }
    }

    if !failed.is_empty() {
        answer.push_str(&format!(
            "\nNote: {} probe{} failed to return data.",
            failed.len(),
            if failed.len() == 1 { "" } else { "s" }
        ));
    }

    if answer.is_empty() {
        format!(
            "I ran {} probe{} but couldn't extract structured information. \
             The raw data is available in the evidence block.",
            probe_results.len(),
            if probe_results.len() == 1 { "" } else { "s" }
        )
    } else {
        answer
    }
}

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
    }
}

/// Fallback context for TRUST+ explanations and v0.0.24 trace-based scoring
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

impl Default for FallbackContext {
    fn default() -> Self {
        Self {
            used_deterministic_fallback: false,
            fallback_route_class: String::new(),
            evidence_kinds: Vec::new(),
            specialist_outcome: None,
            fallback_used: None,
            evidence_required: None,
        }
    }
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

    // Determine answer quality
    let answer_grounded = if used_deterministic {
        parsed_data_count > 0
    } else {
        scoring::check_answer_grounded(answer, probe_results)
    };

    let no_invention = if used_deterministic {
        parsed_data_count > 0
    } else {
        scoring::check_no_invention(answer)
    };

    // Evidence requirement: prefer route capability (v0.45.x probe spine), fall back to heuristic
    let evidence_required = fallback_ctx.evidence_required
        .unwrap_or_else(|| query_requires_evidence(query));

    // Build input for new scoring model (COST: includes resource cap signals)
    // Note: grounding_ratio and total_claims default to 0 for now (legacy path)
    // Full ANCHOR integration would extract claims and verify against parsed probe data
    let input = ReliabilityInput {
        planned_probes,
        succeeded_probes,
        failed_probes,
        timed_out_probes,
        translator_confidence: ticket.confidence,
        translator_used: !used_deterministic && !translator_timed_out,
        answer_grounded,
        no_invention,
        grounding_ratio: 0.0,  // TODO: integrate with claims extraction
        total_claims: 0,       // TODO: integrate with claims extraction
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
