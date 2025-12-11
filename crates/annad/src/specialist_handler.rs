//! Specialist LLM handler with fallback logic (v0.0.403).
//! v0.0.143: Added streaming LLM support.
//! v0.0.241: Added real-time streaming token output to client.
//! v0.0.290: Use clean_llm_response to strip reasoning tags.
//! v0.0.403: CRITICAL - Try deterministic first BEFORE LLM to avoid dumb model answers.
//!
//! Extracted from rpc_handler to keep file sizes manageable.

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{ProbeResult, RuntimeContext, TranslatorTicket};
use anna_shared::trace::SpecialistOutcome;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
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
/// v0.0.403: Now tries DIRECT probe answer FIRST, before LLM
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

    // v0.0.403: CRITICAL - Try direct probe answer FIRST before LLM
    // The LLM is too dumb and says "I don't have that data" even when data is present
    if let Some(direct_result) = probe_direct::try_direct_answer(query, probe_results) {
        info!(
            "v0.0.403: Direct probe answer bypassed LLM (confidence={})",
            direct_result.confidence
        );
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

    // Use summarized probe context (not raw output)
    let probe_context = summarizer::build_probe_context(probe_results);
    let prompt_result =
        service_desk::build_specialist_prompt(ticket.domain, context, probe_results);

    // COST: Log if prompt was truncated
    if prompt_result.was_truncated {
        if let Some(diag) = &prompt_result.diagnostic {
            warn!("COST: {}", diag.format());
        }
    }

    // Only include raw output if debug mode AND explicitly requested
    let full_prompt = if debug_mode && query.to_lowercase().contains("show raw") {
        format!(
            "{}\n\nProbe Output:\n{}\n\nUser: {}",
            prompt_result.prompt, probe_context, query
        )
    } else {
        format!("{}\n\nUser: {}", prompt_result.prompt, query)
    };

    // v0.0.30: Enforce prompt size cap - skip to fallback if prompt too large
    if full_prompt.len() > config.max_specialist_prompt_bytes {
        warn!(
            "Specialist prompt exceeds cap ({}B > {}B), using fallback",
            full_prompt.len(),
            config.max_specialist_prompt_bytes
        );
        progress.skip_stage_deterministic(RequestStage::Specialist);
        let (ans, used_det, det) =
            try_deterministic_fallback(query, context, probe_results, progress);
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

    // v0.0.143: Use streaming LLM with token counting
    // v0.0.241: Also emit streaming tokens for real-time client display
    let token_count = Arc::new(AtomicUsize::new(0));
    let token_count_clone = Arc::clone(&token_count);
    let streaming_sink = progress.streaming_sink();

    let (answer, used_deterministic, det_result, outcome, fallback_route_class) = match timeout(
        Duration::from_secs(config.specialist_timeout_secs),
        ollama::chat_streaming_with_retry(
            model,
            &full_prompt,
            config.specialist_timeout_secs,
            move |token| {
                // Count tokens as they stream in
                token_count_clone.fetch_add(1, Ordering::Relaxed);
                // v0.0.241: Push token to client for real-time display
                streaming_sink.push_token(RequestStage::Specialist, token, false);
            },
        ),
    )
    .await
    {
        Ok(Ok(response)) => {
            let final_tokens = token_count.load(Ordering::Relaxed);
            let duration_ms = stage_start.elapsed().as_millis() as u64;
            info!("Specialist generated {} tokens", final_tokens);
            // v0.0.241: Mark streaming as done
            progress.add_streaming_token(RequestStage::Specialist, "", true);
            progress.complete_stage(RequestStage::Specialist);
            // v0.0.290: Clean response (strip reasoning tags + redact sensitive content)
            let cleaned = redact::clean_llm_response(&response);
            progress.add_specialist_message(&cleaned);
            // v0.0.302: Record LLM call details if debug mode
            if debug_mode {
                progress.add_llm_call(
                    "specialist",
                    model,
                    &full_prompt,
                    &response,
                    duration_ms,
                    Some(final_tokens as u32),
                );
            }
            (cleaned, false, None, SpecialistOutcome::Ok, None)
        }
        Ok(Err(e)) => {
            error!("Specialist LLM error: {}", e);
            progress.error_stage(RequestStage::Specialist, &e.to_string());
            let (ans, used_det, det) =
                try_deterministic_fallback(query, context, probe_results, progress);
            let route_class = det.as_ref().map(|d| d.route_class.clone());
            (ans, used_det, det, SpecialistOutcome::Error, route_class)
        }
        Err(_) => {
            warn!("Specialist timeout, trying deterministic fallback");
            progress.timeout_stage(RequestStage::Specialist);
            let (ans, used_det, det) =
                try_deterministic_fallback(query, context, probe_results, progress);
            let route_class = det.as_ref().map(|d| d.route_class.clone());
            (ans, used_det, det, SpecialistOutcome::Timeout, route_class)
        }
    };

    // Record specialist latency
    {
        state
            .write()
            .await
            .latency
            .specialist
            .add(stage_start.elapsed().as_millis() as u64);
    }

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
    if let Some((answer, parsed_count)) =
        crate::answers::generate_best_effort_summary(probe_results)
    {
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
        return (answer, true, Some(det));
    }

    warn!("No fallback could produce answer from available evidence");
    (String::new(), true, None)
}
