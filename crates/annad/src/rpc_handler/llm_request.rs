//! LLM request handling (v0.0.200).
//!
//! v0.0.247: Streaming events shared with daemon state for live polling.
//! v0.0.248: Fix stats tracking - record ALL requests at start, not just completed ones.

use anna_shared::progress::RequestStage;
use anna_shared::rpc::{RequestParams, RpcResponse};
use anna_shared::status::LlmState;
use anna_shared::trace::SpecialistOutcome;
use anna_shared::transcript::TranscriptEvent;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::comms::{team_from_domain, CommsGenerator};
use crate::configure_editor::{handle_configure_editor, ConfigureEditorResult};
use crate::fast_path_handler::{build_fast_path_result, try_fast_path_answer};
use crate::probe_stage::{check_evidence_validity, execute_probe_stage};
use crate::progress_tracker::ProgressTracker;
use crate::recipe_fast_path;
use crate::result_stage::{build_final_result, wrap_with_theatre};
use crate::router;
use crate::routing_stage::{enforce_probe_spine, route_query};
use crate::service_desk;
use crate::specialist_stage::execute_specialist_stage;
use crate::state::SharedState;
use crate::theatre::TheatreContext;
use crate::timeout_handler::make_timeout_response;
use crate::triage;

use super::helpers::{record_event_log, save_progress};

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

    // v0.0.247: Get shared streaming events from state for live polling
    let streaming_events = {
        let state = state.read().await;
        Arc::clone(&state.streaming_events)
    };
    let mut progress = ProgressTracker::with_streaming_events(streaming_events);

    // Get config, models, and hardware from state
    let (llm_config, translator_model, specialist_model, hw_cores, hw_ram_gb, has_gpu, debug_mode) = {
        let state = state.read().await;
        if state.llm.state != LlmState::Ready {
            return RpcResponse::error(id, -32002, format!("LLM not ready: {}", state.llm.state));
        }
        (
            state.config.llm.clone(),
            state.config.llm.translator_model.clone(),
            state.config.llm.specialist_model.clone(),
            state.hardware.cpu_cores,
            state.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            state.hardware.gpu.is_some(),
            state.config.debug_mode(),
        )
    };

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
    {
        state.write().await.progress_events.clear();
    }

    // Step 0: Fast path check (v0.0.39) - answer health/status queries without LLM
    let fast_path_config = {
        let state = state.read().await;
        (
            state.config.daemon.fast_path_enabled,
            state.config.daemon.snapshot_max_age_secs,
        )
    };

    if fast_path_config.0 {
        if let Some(result) = try_fast_path_answer(query, fast_path_config.1) {
            info!(
                "Fast path handled: class={}, reliability={}",
                result.class, result.reliability
            );

            // Add fast path event to transcript
            let elapsed = progress.elapsed_ms();
            progress.transcript_mut().push(TranscriptEvent::fast_path(
                elapsed,
                true,
                result.class.to_string(),
                &result.trace_note,
                false, // No probes needed if we had fresh snapshot
            ));

            // Build result and return immediately
            let fast_result = build_fast_path_result(
                request_id,
                result.answer,
                result.class,
                result.reliability,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(fast_result).unwrap());
        }
    }

    // Step 1: Deterministic routing (always runs)
    let det_route = router::get_route(query);
    info!(
        "Router: class={:?}, domain={}, probes={:?}",
        det_route.class, det_route.domain, det_route.probes
    );

    // Step 2: v0.0.167 - Route query through recipe check or LLM translator
    let routing_result = route_query(
        &state,
        query,
        &det_route,
        &llm_config,
        &translator_model,
        hw_cores,
        hw_ram_gb,
        has_gpu,
        &mut progress,
    )
    .await;

    // Handle recipe direct answer
    if let Some(ref recipe_result) = routing_result.recipe_result {
        if recipe_fast_path::can_answer_directly(recipe_result) {
            let recipe = recipe_result.recipe.as_ref().unwrap();
            progress.add_translator_message(&format!(
                "Recipe match: {} (score {})",
                recipe.id, recipe_result.score
            ));
            let result = recipe_fast_path::build_recipe_result(
                request_id,
                recipe,
                &recipe_result.matched_tokens,
                progress.take_transcript(),
            );
            return wrap_with_theatre(id, result, None);
        }
    }

    let mut ticket = routing_result.ticket;
    let triage_result = routing_result.triage_result;
    let translator_timed_out = routing_result.translator_timed_out;

    // Step 2.5: v0.0.167 - Enforce probe spine constraints
    enforce_probe_spine(&mut ticket, query, &det_route);

    let classified_domain = ticket.domain;
    let ticket_probes_planned = ticket.needs_probes.len();
    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}, confidence={:.2}",
        ticket.domain, ticket.intent, ticket.needs_probes, ticket.confidence
    ));

    // v0.0.148: Create comms generator for fly-on-wall experience
    let team = team_from_domain(&classified_domain.to_string());
    let mut comms = CommsGenerator::new(team, &request_id);

    // v0.0.148: Anna dispatches to team and junior acknowledges
    comms.dispatch(&mut progress);
    comms.junior_ack(&mut progress);
    save_progress(&state, &progress).await;

    // Step 3: Check if immediate clarification needed (from triage)
    if let Some(ref triage) = triage_result {
        if triage.needs_immediate_clarification {
            save_progress(&state, &progress).await;
            let question = triage
                .clarification_question
                .clone()
                .unwrap_or_else(|| triage::generate_heuristic_clarification(query));
            let result = service_desk::create_clarification_response(
                request_id,
                ticket,
                &question,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 4: v0.0.167 - Run probes via probe_stage module
    let probe_cap_warning = triage_result
        .as_ref()
        .map(|t| t.probe_cap_applied)
        .unwrap_or(false);

    let probe_stage_result =
        execute_probe_stage(&state, &ticket, &llm_config, &mut progress, &mut comms).await;

    // Handle probe timeout
    if probe_stage_result.timed_out {
        let result = service_desk::create_timeout_response(
            request_id,
            "probes",
            Some(ticket),
            vec![],
            progress.take_transcript(),
            classified_domain,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    let probe_results = probe_stage_result.results;

    // Step 5: v0.45.7 Evidence enforcement - "no evidence, no claims" rule
    let valid_evidence_count = check_evidence_validity(&probe_results);
    if det_route.capability.evidence_required && valid_evidence_count == 0 {
        info!("v0.45.7: No valid evidence collected but evidence required - returning deterministic failure");
        save_progress(&state, &progress).await;
        let required_evidence: Vec<String> = det_route
            .capability
            .required_evidence
            .iter()
            .map(|k| k.to_string())
            .collect();
        let result = service_desk::create_no_evidence_response(
            request_id,
            ticket,
            probe_results,
            progress.take_transcript(),
            classified_domain,
            &required_evidence,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 5.5: v0.0.149 ConfigureEditor - extracted to separate module
    if det_route.class == router::QueryClass::ConfigureEditor {
        let editor_result = handle_configure_editor(
            request_id.clone(),
            query,
            ticket.clone(),
            &probe_results,
            progress.transcript_clone(),
            classified_domain,
        );

        if let ConfigureEditorResult::Handled(result) = editor_result {
            save_progress(&state, &progress).await;
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 6: Build context with summarized probes
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    // v0.0.148: Junior reviewing the data
    comms.junior_reviewing(&mut progress);
    save_progress(&state, &progress).await;

    // Step 7: v0.0.167 - Execute specialist stage via module
    let specialist_result = execute_specialist_stage(
        &state,
        query,
        &context,
        &probe_results,
        &ticket,
        &det_route,
        &llm_config,
        &specialist_model,
        debug_mode,
        &mut progress,
    )
    .await;

    // Step 8: Handle no answer case
    if specialist_result.answer.is_empty() {
        save_progress(&state, &progress).await;
        let result = service_desk::create_no_data_response(
            request_id,
            ticket,
            probe_results,
            progress.take_transcript(),
            classified_domain,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 9: Build final result with proper scoring
    progress.start_stage(RequestStage::Supervisor, llm_config.supervisor_timeout_secs);
    progress.add_final_answer(&specialist_result.answer);

    // v0.0.148: Junior confirms answer or escalates based on outcome
    match specialist_result.outcome {
        SpecialistOutcome::Ok | SpecialistOutcome::Skipped => {
            let approx_confidence = if specialist_result.used_deterministic {
                85
            } else {
                75
            };
            comms.junior_done(&mut progress, approx_confidence);
        }
        SpecialistOutcome::Timeout | SpecialistOutcome::Error => {
            comms.junior_escalate(&mut progress, "LLM had trouble, used fallback");
            comms.senior_response(&mut progress, specialist_result.used_deterministic);
        }
        SpecialistOutcome::BudgetExceeded => {
            comms.junior_escalate(&mut progress, "Query too complex");
            comms.senior_response(&mut progress, false);
        }
    }
    comms.anna_returning(&mut progress);
    save_progress(&state, &progress).await;

    // v0.0.166: Use result_stage module for final result building
    let result = build_final_result(
        request_id,
        query,
        ticket,
        probe_results.clone(),
        progress.transcript_clone(),
        classified_domain,
        translator_timed_out,
        &specialist_result,
        det_route.capability.evidence_required,
        ticket_probes_planned,
        probe_cap_warning,
    );

    progress.complete_stage(RequestStage::Supervisor);

    // Record total request latency
    let total_ms = request_start.elapsed().as_millis() as u64;
    {
        let mut state = state.write().await;
        state.latency.total.add(total_ms);
        // v0.0.79: Record stats
        let specialist_timeout = matches!(specialist_result.outcome, SpecialistOutcome::Timeout);
        state.record_request(
            specialist_result.used_deterministic,
            translator_timed_out,
            specialist_timeout,
        );
    }

    info!(
        "Request completed: domain={}, reliability={}, deterministic={}, trace={}, latency={}ms",
        result.domain,
        result.reliability_score,
        specialist_result.used_deterministic,
        result
            .execution_trace
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_default(),
        total_ms
    );

    save_progress(&state, &progress).await;

    // v0.0.106: Create theatre context for Service Desk Theatre
    let mut theatre = TheatreContext::new(query, classified_domain);
    theatre.start_work();

    // v0.0.108: Escalate to senior if reliability is low
    if result.reliability_score < 60 && !result.needs_clarification {
        theatre.escalate();
    }

    theatre.resolve(result.answer.clone(), result.reliability_score, total_ms);

    // v0.0.107: Record topic to user profile for personalized greetings
    theatre.record_topic_to_profile();

    // v0.0.107: Record staff performance metrics
    theatre.record_staff_stats(result.reliability_score, total_ms);

    // v0.0.169: Record event to event log for gamification stats persistence
    record_event_log(&id, &result, &theatre, total_ms);

    // v0.0.166: Return with theatre context using result_stage module
    wrap_with_theatre(id, result, Some(theatre))
}
