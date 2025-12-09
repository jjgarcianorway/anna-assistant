//! Specialist LLM execution stage for the RPC handler pipeline.
//!
//! Extracted from rpc_handler.rs (v0.0.165) for modularization.

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{ProbeResult, RuntimeContext, TranslatorTicket};
use anna_shared::trace::SpecialistOutcome;
use tracing::{info, warn};

use crate::config::LlmConfig;
use crate::deterministic;
use crate::progress_tracker::ProgressTracker;
use crate::router::DeterministicRoute;
use crate::specialist_handler::{try_specialist_llm, SpecialistResult};
use crate::state::SharedState;

/// Execute specialist stage - tries deterministic answer first, then LLM
pub async fn execute_specialist_stage(
    state: &SharedState,
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
    ticket: &TranslatorTicket,
    det_route: &DeterministicRoute,
    llm_config: &LlmConfig,
    specialist_model: &str,
    debug_mode: bool,
    progress: &mut ProgressTracker,
) -> SpecialistResult {
    // Try deterministic answer FIRST for known query classes
    if det_route.can_answer_deterministically() {
        if let Some(det) = deterministic::try_answer(query, context, probe_results) {
            if det.parsed_data_count > 0 {
                info!(
                    "Deterministic answer produced ({} entries)",
                    det.parsed_data_count
                );
                // Skip specialist stage - deterministic router answered
                progress.skip_stage_deterministic(RequestStage::Specialist);
                let route_class = det.route_class.clone();
                return SpecialistResult {
                    answer: det.answer.clone(),
                    used_deterministic: true,
                    det_result: Some(det),
                    prompt_truncated: false,
                    outcome: SpecialistOutcome::Skipped,
                    fallback_route_class: Some(route_class),
                };
            } else {
                warn!("Deterministic parser produced empty result");
            }
        }
    }

    // Fall back to LLM specialist
    try_specialist_llm(
        state,
        query,
        context,
        probe_results,
        ticket,
        llm_config,
        specialist_model,
        debug_mode,
        progress,
    )
    .await
}
