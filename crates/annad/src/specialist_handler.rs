//! Specialist LLM handler with fallback logic (v0.0.39).
//!
//! Extracted from rpc_handler to keep file sizes manageable.

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{ProbeResult, RuntimeContext, TranslatorTicket};
use anna_shared::trace::SpecialistOutcome;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

use crate::config::LlmConfig;
use crate::deterministic::{self, DeterministicResult};
use crate::ollama;
use crate::progress_tracker::ProgressTracker;
use crate::redact;
use crate::service_desk;
use crate::state::SharedState;
use crate::summarizer;

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
pub async fn try_specialist_llm(
    state: &SharedState,
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
    ticket: &TranslatorTicket,
    config: &LlmConfig,
    model: &str,
    debug_mode: bool,
    progress: &mut ProgressTracker,
) -> SpecialistResult {
    progress.start_stage(RequestStage::Specialist, config.specialist_timeout_secs);
    let stage_start = std::time::Instant::now();

    // Use summarized probe context (not raw output)
    let probe_context = summarizer::build_probe_context(probe_results);
    let prompt_result = service_desk::build_specialist_prompt(ticket.domain, context, probe_results);

    // COST: Log if prompt was truncated
    if prompt_result.was_truncated {
        if let Some(diag) = &prompt_result.diagnostic {
            warn!("COST: {}", diag.format());
        }
    }

    // Only include raw output if debug mode AND explicitly requested
    let full_prompt = if debug_mode && query.to_lowercase().contains("show raw") {
        format!("{}\n\nProbe Output:\n{}\n\nUser: {}", prompt_result.prompt, probe_context, query)
    } else {
        format!("{}\n\nUser: {}", prompt_result.prompt, query)
    };

    // v0.0.30: Enforce prompt size cap - skip to fallback if prompt too large
    if full_prompt.len() > config.max_specialist_prompt_bytes {
        warn!("Specialist prompt exceeds cap ({}B > {}B), using fallback",
            full_prompt.len(), config.max_specialist_prompt_bytes);
        progress.skip_stage_deterministic(RequestStage::Specialist);
        let (ans, used_det, det) = try_deterministic_fallback(query, context, probe_results, progress);
        let route_class = det.as_ref().map(|d| d.route_class.clone());
        return SpecialistResult {
            answer: ans,
            used_deterministic: used_det,
            det_result: det,
            prompt_truncated: true,
            outcome: SpecialistOutcome::BudgetExceeded,
            fallback_route_class: route_class,
        };
    }

    let (answer, used_deterministic, det_result, outcome, fallback_route_class) = match timeout(
        Duration::from_secs(config.specialist_timeout_secs),
        ollama::chat_with_timeout(model, &full_prompt, config.specialist_timeout_secs),
    ).await {
        Ok(Ok(response)) => {
            progress.complete_stage(RequestStage::Specialist);
            // Redact sensitive content from response
            let redacted = redact::redact(&response);
            progress.add_specialist_message(&redacted);
            (redacted, false, None, SpecialistOutcome::Ok, None)
        }
        Ok(Err(e)) => {
            error!("Specialist LLM error: {}", e);
            progress.error_stage(RequestStage::Specialist, &e.to_string());
            let (ans, used_det, det) = try_deterministic_fallback(query, context, probe_results, progress);
            let route_class = det.as_ref().map(|d| d.route_class.clone());
            (ans, used_det, det, SpecialistOutcome::Error, route_class)
        }
        Err(_) => {
            warn!("Specialist timeout, trying deterministic fallback");
            progress.timeout_stage(RequestStage::Specialist);
            let (ans, used_det, det) = try_deterministic_fallback(query, context, probe_results, progress);
            let route_class = det.as_ref().map(|d| d.route_class.clone());
            (ans, used_det, det, SpecialistOutcome::Timeout, route_class)
        }
    };

    // Record specialist latency
    { state.write().await.latency.specialist.add(stage_start.elapsed().as_millis() as u64); }

    SpecialistResult {
        answer,
        used_deterministic,
        det_result,
        prompt_truncated: prompt_result.was_truncated,
        outcome,
        fallback_route_class,
    }
}

/// Try deterministic fallback after LLM failure
/// v0.0.30: Now uses best-effort summary from evidence when query-based fallback fails
pub fn try_deterministic_fallback(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
    progress: &mut ProgressTracker,
) -> (String, bool, Option<DeterministicResult>) {
    // First try query-based deterministic answer
    match deterministic::try_answer(query, context, probe_results) {
        Some(det) if det.parsed_data_count > 0 => {
            info!("Deterministic fallback produced answer");
            progress.add_specialist_message("[deterministic fallback]");
            return (det.answer.clone(), true, Some(det));
        }
        _ => {}
    }

    // v0.0.30: If query-based fallback fails, try best-effort summary from evidence
    if let Some((answer, parsed_count)) = crate::answers::generate_best_effort_summary(probe_results) {
        info!("Best-effort summary produced from {} evidence pieces", parsed_count);
        progress.add_specialist_message("[best-effort fallback]");
        let det = DeterministicResult {
            answer: answer.clone(),
            grounded: true,
            parsed_data_count: parsed_count,
            route_class: "best_effort".to_string(),
        };
        return (answer, true, Some(det));
    }

    warn!("No fallback could produce answer from available evidence");
    (String::new(), true, None)
}
