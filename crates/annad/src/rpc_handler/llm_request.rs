//! LLM request handling (v0.0.291).
//!
//! v0.0.247: Streaming events shared with daemon state for live polling.
//! v0.0.248: Fix stats tracking - record ALL requests at start, not just completed ones.
//! v0.0.254: LLM-powered natural dialogue for specialist chatter.
//! v0.0.290: Integrated ticket verification loop for proper Junior->Senior escalation.
//! v0.0.291: Extracted verification_stage for modularization.
//! v0.0.291: Split into smaller modules (fast_path_stage, deterministic_handlers, formatting).

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{RequestParams, RpcResponse};
use anna_shared::status::LlmState;
use anna_shared::trace::SpecialistOutcome;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::progress_tracker::ProgressTracker;
use crate::result_stage::wrap_with_theatre;
use crate::router;
use crate::specialist_stage::execute_specialist_stage;
use crate::state::SharedState;
use crate::timeout_handler::make_timeout_response;

use super::deterministic_handlers::{try_all_deterministic_handlers, DeterministicHandlerResult};
use super::fast_path_stage::try_handle_fast_path;
use super::formatting::format_deterministic_answer;
use super::helpers::save_progress;
use super::probe_handler::{check_and_handle_evidence, execute_and_handle_probes};
use super::request_helpers::{
    create_no_data_response, extract_config, extract_fast_path_config, log_request_completion,
    record_request_stats, save_truth_ledger,
};
use super::routing_handler::{handle_routing_stage, handle_team_comms};
use super::triage_handler::check_and_handle_clarification;
use super::verification_stage::{self, VerificationInput};

/// Service desk pipeline with deterministic routing, triage, and fallback
pub async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let request_id = uuid::Uuid::new_v4().to_string();

    // v0.0.248: Record request immediately to track ALL requests including failures
    {
        state.write().await.stats.record_request_received();
    }

    let request_timeout = { state.read().await.config.daemon.request_timeout_secs };

    // Extract query for timeout fallback (v0.0.40)
    let query_for_fallback = params
        .as_ref()
        .and_then(|p| p.get("prompt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match timeout(
        Duration::from_secs(request_timeout),
        handle_llm_request_inner(state.clone(), id.clone(), params, request_id.clone()),
    )
    .await
    {
        Ok(response) => response,
        Err(_) => {
            warn!("Global request timeout ({}s)", request_timeout);
            make_timeout_response(
                id,
                request_id,
                request_timeout,
                query_for_fallback.as_deref(),
            )
        }
    }
}

/// Inner request handler (wrapped by global timeout)
async fn handle_llm_request_inner(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
    request_id: String,
) -> RpcResponse {
    let request_start = Instant::now();

    // v0.0.266: Clear old progress events BEFORE creating new tracker to prevent leaking
    // This fixes the bug where events from previous request appeared at start of new request
    {
        state.write().await.progress_events.clear();
    }

    // v0.0.247: Get shared streaming events from state for live polling
    let streaming_events = {
        let state = state.read().await;
        Arc::clone(&state.streaming_events)
    };
    let mut progress = ProgressTracker::with_streaming_events(streaming_events);

    // Get config, models, and hardware from state
    // v0.0.310: Allow requests when models are loading - deterministic answers work
    let config = match extract_config(&state, &id).await {
        Ok(c) => c,
        Err(response) => return response,
    };

    // v0.0.310: Log if serving while models are still loading
    if !config.models_fully_ready {
        info!("Serving request while models are loading (deterministic answers only)");
    }

    // Parse parameters
    let params: RequestParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => return RpcResponse::error(id, -32602, format!("Invalid params: {}", e)),
        },
        None => return RpcResponse::error(id, -32602, "Missing params".to_string()),
    };

    let query = &params.prompt;
    progress.add_user_message(query);
    // v0.0.266: progress_events.clear() moved to start of function to prevent context leaking

    // Step 0: Fast path check (v0.0.39) - answer health/status queries without LLM
    let fast_path_config = extract_fast_path_config(&state).await;

    if let Some(response) = try_handle_fast_path(
        &id,
        &request_id,
        query,
        fast_path_config.enabled,
        fast_path_config.snapshot_max_age_secs,
        &mut progress,
    ) {
        return response;
    }

    // Step 1: Deterministic routing (always runs)
    let det_route = router::get_route(query);
    info!(
        "Router: class={:?}, domain={}, probes={:?}",
        det_route.class, det_route.domain, det_route.probes
    );

    // Step 2: Handle routing stage (recipe check + LLM translator)
    let (recipe_response, ticket, routing_result) = handle_routing_stage(
        &state,
        &id,
        &request_id,
        query,
        &det_route,
        &config,
        &mut progress,
    )
    .await;

    if let Some(response) = recipe_response {
        return response;
    }

    let triage_result = routing_result.triage_result.clone();
    let translator_timed_out = routing_result.translator_timed_out;

    let classified_domain = ticket.domain;
    let ticket_probes_planned = ticket.needs_probes.len();
    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}, confidence={:.2}",
        ticket.domain, ticket.intent, ticket.needs_probes, ticket.confidence
    ));

    // Step 2.5: Handle team communications
    let mut comms = handle_team_comms(
        &det_route.class.to_string(),
        &classified_domain.to_string(),
        &request_id,
        query,
        &mut progress,
        &state,
    )
    .await;

    // Step 3: Check if immediate clarification needed (from triage)
    if let Some(response) = check_and_handle_clarification(
        &id,
        &request_id,
        query,
        &routing_result,
        &ticket,
        classified_domain,
        &state,
        &mut progress,
    )
    .await
    {
        return response;
    }

    // Step 4: v0.0.167 - Run probes via probe_stage module
    let probe_cap_warning = triage_result
        .as_ref()
        .map(|t| t.probe_cap_applied)
        .unwrap_or(false);

    let (probe_results, probe_timeout) = execute_and_handle_probes(
        &id,
        &request_id,
        &state,
        &ticket,
        &config,
        &mut progress,
        &mut comms,
        classified_domain,
    )
    .await;

    if let Some(response) = probe_timeout {
        return response;
    }

    // Step 5: v0.45.7 Evidence enforcement - "no evidence, no claims" rule
    if let Some(response) = check_and_handle_evidence(
        &id,
        &request_id,
        &det_route,
        &ticket,
        &probe_results,
        &mut progress,
        classified_domain,
        &state,
    )
    .await
    {
        return response;
    }

    // Step 5.5-5.8: Try all deterministic handlers (ConfigureEditor, DesktopWallpaper, etc.)
    if let DeterministicHandlerResult::Handled(response) = try_all_deterministic_handlers(
        &id,
        &request_id,
        &det_route.class,
        query,
        &ticket,
        &probe_results,
        progress.transcript_clone(),
        classified_domain,
    ) {
        save_progress(&state, &progress).await;
        return response;
    }

    // Step 6: Build context with summarized probes
    let context = {
        let state_read = state.read().await;
        super::request_helpers::build_context(&state_read.hardware, &probe_results)
    };

    // v0.0.148: Junior reviewing the data
    comms.junior_reviewing_async(&mut progress).await;
    save_progress(&state, &progress).await;

    // Step 7: v0.0.167 - Execute specialist stage via module
    let mut specialist_result = {
        let state_read = state.read().await; // Acquire read lock once
        let truth_ledger_ref = &state_read.truth_ledger; // Get reference
        execute_specialist_stage(
            &state,
            query,
            &context,
            &probe_results,
            &ticket,
            &det_route,
            &config.llm_config,
            &config.specialist_model,
            config.debug_mode,
            &mut progress,
            truth_ledger_ref, // Pass truth_ledger_ref
        )
        .await
    };

    // Step 7.5: v0.0.276 - Format deterministic answers via translator LLM
    // v0.0.794: Skip formatting for data-listing answers (ports, services, etc.)
    specialist_result.answer = format_deterministic_answer(
        specialist_result.answer,
        specialist_result.used_deterministic,
        specialist_result.det_result.as_ref(),
        &config.translator_model,
        query,
    )
    .await;

    // Step 8: Handle no answer case
    if specialist_result.answer.is_empty() {
        save_progress(&state, &progress).await;
        return create_no_data_response(
            &id,
            request_id,
            ticket,
            probe_results,
            progress.transcript_clone(),
            classified_domain,
        );
    }

    // Step 9: v0.0.297 - Run verification stage with LLM self-healing
    let verification_input = VerificationInput {
        request_id: &request_id,
        query,
        specialist_result: &specialist_result,
        ticket: &ticket,
        probe_results: &probe_results,
        det_route: &det_route,
        classified_domain,
        translator_timed_out,
        ticket_probes_planned,
        probe_cap_warning,
        supervisor_timeout_secs: config.llm_config.supervisor_timeout_secs,
        model: &config.specialist_model,
    };

    // v0.0.298: run_verification returns VerificationResult with validated status
    let verification_result = verification_stage::run_verification(
        &state,
        &verification_input,
        &mut progress,
        &mut comms,
    )
    .await;
    save_progress(&state, &progress).await;

    progress.add_final_answer(&verification_result.answer);

    // Build final result with verified answer (v0.0.298: pass validated status)
    let result = verification_stage::build_verified_result(
        &verification_input,
        verification_result.answer.clone(),
        progress.transcript_clone(),
        verification_result.validated,
    );

    progress.complete_stage(RequestStage::Supervisor);

    // Record total request latency
    let total_ms = request_start.elapsed().as_millis() as u64;
    let specialist_timeout = matches!(specialist_result.outcome, SpecialistOutcome::Timeout);
    record_request_stats(
        &state,
        total_ms,
        specialist_result.used_deterministic,
        translator_timed_out,
        specialist_timeout,
    )
    .await;

    log_request_completion(
        &result.domain.to_string(),
        result.reliability_score,
        specialist_result.used_deterministic,
        result.execution_trace.as_ref().map(|t| t.to_string()),
        total_ms,
    );

    save_progress(&state, &progress).await;
    save_truth_ledger(&state).await;

    // v0.0.291: Handle theatre recording via extracted module
    let theatre = verification_stage::handle_theatre(
        query,
        classified_domain,
        &result,
        &id,
        total_ms,
        result.validated,
    );

    // Return with theatre context
    wrap_with_theatre(id, result, Some(theatre))
}
