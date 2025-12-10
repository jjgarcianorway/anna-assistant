//! Clarification response builders for ServiceDeskResult.
//!
//! Extracted from service_desk.rs (v0.0.156) for modularization.
//! Contains builders for clarification responses with and without options.

use anna_shared::clarify_v2::{ClarifyOption, ClarifyRequest};
use anna_shared::parsers::parse_probe_result;
use anna_shared::rpc::{
    EvidenceBlock, ProbeResult, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
    TranslatorTicket,
};
use anna_shared::trace::{evidence_kinds_from_probes, ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;
use uuid::Uuid;

use crate::service_desk::{get_relevant_hardware_fields, CLARIFICATION_MAX_RELIABILITY};

/// Create a clarification response (legacy - no probes attached)
pub fn create_clarification_response(
    request_id: String,
    ticket: TranslatorTicket,
    question: &str,
    transcript: Transcript,
) -> ServiceDeskResult {
    create_clarification_response_grounded(
        request_id,
        ticket,
        question,
        vec![],
        transcript,
        false, // Not grounded when no probes
    )
}

/// Create a grounded clarification response with probes attached (v0.0.59).
/// Use this when clarification options were derived from current probe evidence.
pub fn create_clarification_response_grounded(
    request_id: String,
    ticket: TranslatorTicket,
    question: &str,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    grounded: bool,
) -> ServiceDeskResult {
    let has_probes = !probe_results.is_empty();
    let signals = ReliabilitySignals {
        translator_confident: has_probes,
        probe_coverage: has_probes,
        answer_grounded: grounded,
        no_invention: true,
        clarification_not_needed: false,
    };
    let score = signals.score().min(CLARIFICATION_MAX_RELIABILITY);

    let evidence = EvidenceBlock {
        hardware_fields: get_relevant_hardware_fields(&ticket),
        probes_executed: probe_results,
        translator_ticket: ticket,
        last_error: None,
    };

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: String::new(),
        validated: false, // v0.0.298: Clarification responses are not validated answers
        reliability_score: score,
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(question.to_string()),
        clarification_request: None,
        transcript,
        execution_trace: None,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// v0.0.59: Create clarification response with structured options from probe evidence.
/// v0.0.62: Now properly accounts probes and sets execution_trace for grounded output.
/// This is the proper way to handle multiple-choice scenarios like ConfigureEditor.
pub fn create_clarification_with_options(
    request_id: String,
    ticket: TranslatorTicket,
    question: &str,
    options: Vec<(String, String)>, // (label, value) pairs
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
) -> ServiceDeskResult {
    // v0.0.62: Count valid evidence from probes
    let valid_evidence_count = probe_results
        .iter()
        .filter(|p| parse_probe_result(p).is_valid_evidence())
        .count();
    let has_valid_evidence = valid_evidence_count > 0;

    let signals = ReliabilitySignals {
        translator_confident: has_valid_evidence,
        probe_coverage: has_valid_evidence,
        answer_grounded: has_valid_evidence,
        no_invention: true,
        clarification_not_needed: false,
    };
    let score = signals.score().min(CLARIFICATION_MAX_RELIABILITY);

    // Build structured ClarifyRequest with options
    let clarify_options: Vec<ClarifyOption> = options
        .iter()
        .enumerate()
        .map(|(i, (label, value))| ClarifyOption::new((i + 1) as u8, label, value))
        .collect();

    let clarify_request = ClarifyRequest {
        id: Uuid::new_v4().to_string(),
        question: question.to_string(),
        options: clarify_options,
        allow_custom: false,
        allow_cancel: true,
        reason: Some("Multiple options detected from system probes".to_string()),
        ttl_seconds: 300,
    };

    let evidence = EvidenceBlock {
        hardware_fields: get_relevant_hardware_fields(&ticket),
        probes_executed: probe_results.clone(),
        translator_ticket: ticket.clone(),
        last_error: None,
    };

    // v0.0.62: Build execution trace with probe stats
    let planned_probes = ticket.needs_probes.len();
    let probe_stats = ProbeStats::from_results(planned_probes, &probe_results);
    let evidence_kinds = evidence_kinds_from_probes(&probe_results);

    let execution_trace = Some(ExecutionTrace {
        specialist_outcome: anna_shared::trace::SpecialistOutcome::Skipped,
        fallback_used: anna_shared::trace::FallbackUsed::None,
        probe_stats,
        evidence_kinds,
        answer_is_deterministic: true,
        reviewer_outcome: None,
    });

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer: String::new(),
        validated: false, // v0.0.298: Clarification responses are not validated answers
        reliability_score: score,
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence,
        needs_clarification: true,
        clarification_question: Some(question.to_string()),
        clarification_request: Some(clarify_request),
        transcript,
        execution_trace,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}
