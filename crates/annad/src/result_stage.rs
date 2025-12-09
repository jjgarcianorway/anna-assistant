//! Result building stage for the RPC handler pipeline.
//!
//! Extracted from rpc_handler.rs (v0.0.165) for modularization.

use anna_shared::rpc::{ProbeResult, RpcResponse, ServiceDeskResult, SpecialistDomain, TranslatorTicket};
use anna_shared::trace::{evidence_kinds_from_probes, ExecutionTrace, ProbeStats, SpecialistOutcome};
use anna_shared::transcript::Transcript;
use tracing::debug;

use crate::service_desk::{self, FallbackContext};
use crate::specialist_handler::SpecialistResult;
use crate::theatre::TheatreContext;

/// Build the final ServiceDeskResult from specialist output
pub fn build_final_result(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: Vec<ProbeResult>,
    transcript: Transcript,
    classified_domain: SpecialistDomain,
    translator_timed_out: bool,
    specialist_result: &SpecialistResult,
    det_route_evidence_required: bool,
    ticket_probes_planned: usize,
    probe_cap_warning: bool,
) -> ServiceDeskResult {
    let SpecialistResult {
        answer,
        used_deterministic,
        det_result,
        prompt_truncated,
        outcome,
        fallback_route_class,
    } = specialist_result;

    let parsed_data_count = det_result.as_ref().map(|d| d.parsed_data_count).unwrap_or(0);

    // Build fallback context for TRUST+ explanations
    let used_deterministic_fallback = matches!(
        outcome,
        SpecialistOutcome::Timeout | SpecialistOutcome::Error
    ) && *used_deterministic
        && parsed_data_count > 0;

    let fallback_used = if used_deterministic_fallback {
        Some(anna_shared::trace::FallbackUsed::Deterministic {
            route_class: fallback_route_class.clone().unwrap_or_default(),
        })
    } else {
        Some(anna_shared::trace::FallbackUsed::None)
    };

    // Derive evidence kinds from ACTUAL probes
    let actual_evidence_kinds = evidence_kinds_from_probes(&probe_results);

    let fallback_ctx = FallbackContext {
        used_deterministic_fallback,
        fallback_route_class: fallback_route_class.clone().unwrap_or_default(),
        evidence_kinds: actual_evidence_kinds
            .iter()
            .map(|k| k.to_string())
            .collect(),
        specialist_outcome: Some(*outcome),
        fallback_used,
        evidence_required: Some(det_route_evidence_required),
    };

    let mut result = service_desk::build_result_with_flags(
        request_id,
        answer.clone(),
        query,
        ticket,
        probe_results.clone(),
        transcript,
        classified_domain,
        translator_timed_out,
        *used_deterministic,
        parsed_data_count,
        *prompt_truncated,
        fallback_ctx,
    );

    // Build execution trace
    let probe_stats = ProbeStats::from_results(ticket_probes_planned, &probe_results);
    let evidence_kinds = actual_evidence_kinds;

    result.execution_trace = Some(match outcome {
        SpecialistOutcome::Skipped => {
            ExecutionTrace::deterministic_route(
                fallback_route_class.as_deref().unwrap_or("unknown"),
                probe_stats,
                evidence_kinds,
            )
        }
        SpecialistOutcome::Ok => ExecutionTrace::specialist_ok(probe_stats),
        SpecialistOutcome::Timeout => {
            if *used_deterministic && parsed_data_count > 0 {
                ExecutionTrace::specialist_timeout_with_fallback(
                    fallback_route_class.as_deref().unwrap_or("unknown"),
                    probe_stats,
                    evidence_kinds,
                )
            } else {
                ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
            }
        }
        SpecialistOutcome::Error => {
            if *used_deterministic && parsed_data_count > 0 {
                ExecutionTrace::specialist_error_with_fallback(
                    fallback_route_class.as_deref().unwrap_or("unknown"),
                    probe_stats,
                    evidence_kinds,
                )
            } else {
                ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
            }
        }
        SpecialistOutcome::BudgetExceeded => {
            ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
        }
    });

    // Add probe cap warning to evidence
    if probe_cap_warning {
        result.evidence.last_error = Some("probe_cap_applied".to_string());
    }

    result
}

/// Wrap result with theatre context for Service Desk Theatre
pub fn wrap_with_theatre(
    id: String,
    mut result: ServiceDeskResult,
    theatre: Option<TheatreContext>,
) -> RpcResponse {
    if let Some(ctx) = theatre {
        result.case_number = Some(ctx.case_number.clone());
        result.assigned_staff = Some(ctx.staff_display());
        result.staff_id = Some(ctx.staff.person_id.to_string());

        // Save ticket to history
        if let Err(e) = ctx.save() {
            debug!("Failed to save ticket to history: {}", e);
        }
    }

    // Try to learn recipe from result
    let learn_result = anna_shared::recipe_learning::try_learn_from_result(&result);
    if learn_result.learned {
        if let Some(recipe_id) = &learn_result.recipe_id {
            debug!("Learned recipe {} from result", recipe_id);
        }
    }

    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}
