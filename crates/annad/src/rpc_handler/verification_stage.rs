//! Verification and theatre stage (v0.0.298).
//!
//! Handles ticket verification loop, comms updates, and theatre recording.
//! Extracted from llm_request.rs for modularization.
//! v0.0.297: LLM self-healing integration via ticket_loop.
//! v0.0.298: Return `validated` status from verification loop.

use anna_shared::progress::RequestStage;
use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::{ServiceDeskResult, SpecialistDomain};
use anna_shared::ticket::TicketStatus;
use anna_shared::trace::SpecialistOutcome;
use anna_shared::transcript::Transcript;

use crate::comms::CommsGenerator;
use crate::progress_tracker::ProgressTracker;
use crate::result_stage::build_final_result;
use crate::router::DeterministicRoute;
use crate::specialist_handler::SpecialistResult;
use crate::theatre::TheatreContext;
use crate::ticket_loop::run_ticket_loop;
use crate::ticket_service::TicketServiceConfig;

use super::helpers::record_event_log;

/// Input for the verification stage
pub struct VerificationInput<'a> {
    pub request_id: &'a str,
    pub query: &'a str,
    pub specialist_result: &'a SpecialistResult,
    pub ticket: &'a anna_shared::rpc::TranslatorTicket,
    pub probe_results: &'a [anna_shared::rpc::ProbeResult],
    pub det_route: &'a DeterministicRoute,
    pub classified_domain: SpecialistDomain,
    pub translator_timed_out: bool,
    pub ticket_probes_planned: usize,
    pub probe_cap_warning: bool,
    pub supervisor_timeout_secs: u64,
    /// v0.0.297: Model for LLM self-healing
    pub model: &'a str,
}

/// Build reliability input for verification
pub fn build_reliability_input(
    input: &VerificationInput,
) -> ReliabilityInput {
    ReliabilityInput {
        planned_probes: input.ticket_probes_planned,
        succeeded_probes: input.probe_results.iter().filter(|p| p.exit_code == 0).count(),
        failed_probes: input.probe_results.iter().filter(|p| p.exit_code != 0).count(),
        timed_out_probes: 0,
        translator_confidence: input.ticket.confidence,
        translator_used: !input.translator_timed_out,
        answer_grounded: true,
        no_invention: true,
        grounding_ratio: 1.0,
        total_claims: 1,
        evidence_required: input.det_route.capability.evidence_required,
        used_deterministic: input.specialist_result.used_deterministic,
        parsed_data_count: input.specialist_result.det_result.as_ref().map(|d| d.parsed_data_count).unwrap_or(0),
        prompt_truncated: input.specialist_result.prompt_truncated,
        transcript_capped: false,
        budget_exceeded: matches!(input.specialist_result.outcome, SpecialistOutcome::BudgetExceeded),
        exceeded_stage: None,
        stage_budget_ms: 0,
        stage_elapsed_ms: 0,
        used_deterministic_fallback: matches!(input.specialist_result.outcome, SpecialistOutcome::Timeout | SpecialistOutcome::Error) && input.specialist_result.used_deterministic,
        fallback_route_class: input.specialist_result.fallback_route_class.clone().unwrap_or_default(),
        evidence_kinds: vec![],
        specialist_outcome: Some(input.specialist_result.outcome),
        fallback_used: None,
    }
}

/// Verification loop result (v0.0.298: includes validated status)
pub struct VerificationResult {
    pub answer: String,
    pub validated: bool,
}

/// Run the verification loop and update comms (v0.0.297: with LLM self-healing)
/// v0.0.298: Returns VerificationResult with validated status
pub async fn run_verification(
    input: &VerificationInput<'_>,
    progress: &mut ProgressTracker,
    comms: &mut CommsGenerator,
) -> VerificationResult {
    progress.start_stage(RequestStage::Supervisor, input.supervisor_timeout_secs);

    let reliability_input = build_reliability_input(input);
    let route_class = input.specialist_result.fallback_route_class.as_deref().unwrap_or("unknown");
    let elapsed_ms = progress.elapsed_ms();

    // v0.0.297: Pass model and timeout for LLM self-healing
    let verification_result = run_ticket_loop(
        input.request_id,
        input.query,
        &input.specialist_result.answer,
        input.ticket,
        route_class,
        input.probe_results,
        &reliability_input,
        progress.transcript_mut(),
        elapsed_ms,
        Some(TicketServiceConfig::default()),
        input.model,
        input.supervisor_timeout_secs,
    )
    .await;

    // Update comms based on verification result
    // v0.0.305: Escalation messages are already in transcript from ticket_loop,
    // only add senior response for Escalated/Failed status (avoid duplicates)
    match verification_result.ticket.status {
        TicketStatus::Verified => {
            comms.junior_done_async(progress, verification_result.score).await;
        }
        TicketStatus::Escalated => {
            // Escalation message already in transcript from ticket_loop
            comms.senior_response(progress, verification_result.verified);
        }
        TicketStatus::Failed => {
            // Escalation message already in transcript from ticket_loop
            comms.senior_response(progress, false);
        }
        _ => {
            comms.junior_done_async(progress, verification_result.score).await;
        }
    }
    comms.anna_returning_async(progress).await;

    // v0.0.298: Return full verification result including validated status
    VerificationResult {
        answer: verification_result.answer,
        validated: verification_result.verified,
    }
}

/// Build final result with verified answer (v0.0.298: accepts validated status)
pub fn build_verified_result(
    input: &VerificationInput<'_>,
    final_answer: String,
    transcript: Transcript,
    validated: bool,
) -> ServiceDeskResult {
    let specialist_result = SpecialistResult {
        answer: final_answer,
        used_deterministic: input.specialist_result.used_deterministic,
        det_result: input.specialist_result.det_result.clone(),
        prompt_truncated: input.specialist_result.prompt_truncated,
        outcome: input.specialist_result.outcome,
        fallback_route_class: input.specialist_result.fallback_route_class.clone(),
    };

    let mut result = build_final_result(
        input.request_id.to_string(),
        input.query,
        input.ticket.clone(),
        input.probe_results.to_vec(),
        transcript,
        input.classified_domain,
        input.translator_timed_out,
        &specialist_result,
        input.det_route.capability.evidence_required,
        input.ticket_probes_planned,
        input.probe_cap_warning,
    );

    // v0.0.298: Set validated from ticket verification loop
    result.validated = validated;

    result
}

/// Handle theatre recording and notifications
/// v0.0.298: Use `validated` field for escalation/notification decisions
pub fn handle_theatre(
    query: &str,
    domain: SpecialistDomain,
    result: &ServiceDeskResult,
    id: &str,
    total_ms: u64,
) -> TheatreContext {
    let mut theatre = TheatreContext::new(query, domain);
    theatre.start_work();

    // v0.0.298: Escalate to senior if not validated (not just score < 60)
    if !result.validated && !result.needs_clarification {
        theatre.escalate();
    }

    theatre.resolve(result.answer.clone(), result.reliability_score, total_ms);

    // Record topic to user profile
    theatre.record_topic_to_profile();

    // Record staff performance metrics
    theatre.record_staff_stats(result.reliability_score, total_ms);

    // v0.0.298: Use validated field for notifications
    if theatre.should_notify(total_ms) || !result.validated {
        theatre.notify_ticket_created();
        if result.validated {
            theatre.notify_ticket_resolved();
        }
    }

    // Record event to event log
    record_event_log(id, result, &theatre, total_ms);

    theatre
}
