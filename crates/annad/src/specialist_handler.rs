//! Specialist LLM handler (v0.0.406).
//! v0.0.403: Try deterministic first BEFORE LLM to avoid dumb model answers.
//! v0.0.404: JSON-only specialist - LLM outputs JSON, personality added by renderer.
//! v0.0.406: Removed old prose path - JSON-only is now the only architecture.
//! v0.0.410: Evidence pipeline integration - use evidence-enhanced specialist.
//!
//! Extracted from rpc_handler to keep file sizes manageable.

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{ProbeResult, RuntimeContext, TranslatorTicket};
use anna_shared::trace::SpecialistOutcome;
use tracing::info;

use crate::config::LlmConfig;
use crate::deterministic::{self, DeterministicResult};
use crate::progress_tracker::ProgressTracker;
use crate::specialist_json;
use crate::state::SharedState;
use crate::probe_direct;

/// Specialist LLM result with resource tracking
pub struct SpecialistResult {
    pub answer: String,
    pub used_deterministic: bool,
    pub det_result: Option<DeterministicResult>,
    pub prompt_truncated: bool,
    /// Outcome of specialist stage (for trace)
    pub outcome: SpecialistOutcome,
    /// Whether fallback was used and what route class
    pub fallback_route_class: Option<String>,
}

/// Try specialist LLM with summarized probe output
/// v0.0.403: Tries DIRECT probe answer FIRST, before LLM
/// v0.0.406: JSON-only architecture - always uses JSON specialist
#[allow(clippy::too_many_arguments)]
pub async fn try_specialist_llm(
    state: &SharedState,
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
    ticket: &TranslatorTicket,
    config: &LlmConfig,
    model: &str,
    _debug_mode: bool,
    progress: &mut ProgressTracker,
) -> SpecialistResult {
    let stage_start = std::time::Instant::now();

    // v0.0.403: CRITICAL - Try direct probe answer FIRST before LLM
    // The LLM is too dumb and says "I don't have that data" even when data is present
    if let Some(direct_result) = probe_direct::try_direct_answer(query, probe_results) {
        info!(
            "v0.0.403: Direct probe answer bypassed LLM (confidence={})",
            direct_result.confidence
        );
        progress.start_stage(RequestStage::Specialist, config.specialist_timeout_secs);
        progress.complete_stage(RequestStage::Specialist);
        progress.add_specialist_message("[direct probe answer]");

        // Record latency
        {
            state
                .write()
                .await
                .latency
                .specialist
                .add(stage_start.elapsed().as_millis() as u64);
        }

        let det = DeterministicResult {
            answer: direct_result.answer.clone(),
            grounded: true,
            parsed_data_count: 1,
            route_class: "probe_direct".to_string(),
        };

        return SpecialistResult {
            answer: direct_result.answer,
            used_deterministic: true,
            det_result: Some(det),
            prompt_truncated: false,
            outcome: SpecialistOutcome::Ok,
            fallback_route_class: Some("probe_direct".to_string()),
        };
    }

    // v0.0.406: JSON-only specialist path (always used now)
    try_json_specialist(
        state,
        query,
        context,
        probe_results,
        ticket,
        config,
        model,
        progress,
    )
    .await
}

/// JSON-only specialist path
/// Uses strict JSON contract - LLM outputs JSON, personality added by renderer
/// v0.0.406: Falls back to deterministic if JSON specialist fails
/// v0.0.410: Uses evidence pipeline for enhanced context
async fn try_json_specialist(
    state: &SharedState,
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
    ticket: &TranslatorTicket,
    config: &LlmConfig,
    model: &str,
    progress: &mut ProgressTracker,
) -> SpecialistResult {
    let stage_start = std::time::Instant::now();
    progress.start_stage(RequestStage::Specialist, config.specialist_timeout_secs);

    // Generate ticket ID for JSON specialist (use timestamp for uniqueness)
    let ticket_id = format!(
        "DSK-{:04}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() % 10000
    );

    info!(
        "v0.0.410: JSON specialist with evidence for domain={:?}, ticket={}",
        ticket.domain, ticket_id
    );

    // v0.0.410: Use evidence-enhanced specialist (includes docs, recipes, knowledge)
    let result = specialist_json::call_json_specialist_with_evidence(
        ticket,
        &ticket_id,
        query,
        probe_results,
        model,
        config.specialist_timeout_secs,
    )
    .await;

    // Record latency
    {
        state
            .write()
            .await
            .latency
            .specialist
            .add(stage_start.elapsed().as_millis() as u64);
    }

    progress.complete_stage(RequestStage::Specialist);

    // v0.0.410: Handle instant answer (knowledge index hit, no LLM needed)
    if !result.used_llm {
        let answer = specialist_json::get_answer_text(&result);
        info!(
            "v0.0.410: Instant answer from knowledge (no LLM): {}",
            &answer[..answer.len().min(50)]
        );
        progress.add_specialist_message("[knowledge index: instant answer]");

        let det = DeterministicResult {
            answer: answer.clone(),
            grounded: true,
            parsed_data_count: 1,
            route_class: "knowledge_index".to_string(),
        };

        return SpecialistResult {
            answer,
            used_deterministic: true,
            det_result: Some(det),
            prompt_truncated: false,
            outcome: SpecialistOutcome::Ok,
            fallback_route_class: Some("knowledge_index".to_string()),
        };
    }

    // Get rendered answer
    let answer = specialist_json::get_answer_text(&result);

    // Check if we got a valid response
    let (outcome, final_answer, used_det, det_result) = if let Some(ref resp) = result.response {
        if resp.status == anna_shared::specialist_contract::ResponseStatus::Ok {
            info!(
                "JSON specialist: status={:?}, confidence={:.0}%, reliability={}%",
                resp.status,
                resp.confidence * 100.0,
                result.rendered.reliability
            );
            progress.add_specialist_message(&format!(
                "[json specialist: {}]",
                result.rendered.internal_comms
            ));
            (SpecialistOutcome::Ok, answer, false, None)
        } else {
            // JSON specialist couldn't answer - try deterministic fallback
            info!("JSON specialist returned {:?}, trying deterministic", resp.status);
            let (det_answer, det_result) = try_deterministic_fallback(query, context, probe_results, progress);
            if !det_answer.is_empty() {
                (SpecialistOutcome::Ok, det_answer, true, det_result)
            } else {
                // Return the JSON specialist's answer even if status wasn't Ok
                (SpecialistOutcome::Ok, answer, false, None)
            }
        }
    } else {
        // Parse failed - try deterministic fallback
        progress.add_specialist_message("[json specialist: parse failed]");
        let (det_answer, det_result) = try_deterministic_fallback(query, context, probe_results, progress);
        if !det_answer.is_empty() {
            (SpecialistOutcome::Ok, det_answer, true, det_result)
        } else {
            (SpecialistOutcome::Error, String::new(), false, None)
        }
    };

    SpecialistResult {
        answer: final_answer,
        used_deterministic: used_det,
        det_result,
        prompt_truncated: false,
        outcome,
        fallback_route_class: Some("json_specialist".to_string()),
    }
}

/// Try deterministic fallback after LLM failure
/// Uses deterministic::try_answer and best-effort summary as fallbacks
fn try_deterministic_fallback(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
    progress: &mut ProgressTracker,
) -> (String, Option<DeterministicResult>) {
    // First try query-based deterministic answer
    if let Some(det) = deterministic::try_answer(query, context, probe_results) {
        if det.parsed_data_count > 0 {
            info!("Deterministic fallback produced answer");
            progress.add_specialist_message("[deterministic fallback]");
            return (det.answer.clone(), Some(det));
        }
    }

    // v0.0.30: If query-based fallback fails, try best-effort summary from evidence
    if let Some((answer, parsed_count)) = crate::answers::generate_best_effort_summary(probe_results) {
        info!(
            "Best-effort summary produced from {} evidence pieces",
            parsed_count
        );
        progress.add_specialist_message("[best-effort fallback]");
        let det = DeterministicResult {
            answer: answer.clone(),
            grounded: true,
            parsed_data_count: parsed_count,
            route_class: "best_effort".to_string(),
        };
        return (answer, Some(det));
    }

    info!("No fallback could produce answer from available evidence");
    (String::new(), None)
}
