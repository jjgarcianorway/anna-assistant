//! RPC request handlers with deterministic routing, triage, and fallback.

use anna_shared::probe_spine::{enforce_minimum_probes, enforce_spine_probes, probe_to_command};
use anna_shared::progress::RequestStage;
use anna_shared::rpc::{RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use anna_shared::trace::{
    evidence_kinds_from_probes, ExecutionTrace, ProbeStats, SpecialistOutcome,
};
use anna_shared::transcript::TranscriptEvent;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

use crate::config::LlmConfig;
use crate::deterministic::{self, DeterministicResult};
use crate::fast_path_handler::{build_fast_path_result, force_fast_path_fallback, is_health_query, try_fast_path_answer};
use crate::handlers;
use crate::probe_runner;
use crate::progress_tracker::ProgressTracker;
use crate::router;
use crate::service_desk;
use crate::specialist_handler::{try_specialist_llm, SpecialistResult};
use crate::state::SharedState;
use crate::triage::{self, TriageResult};
use crate::translator::{self, TranslatorInput};

/// Handle an RPC request
pub async fn handle_request(state: SharedState, request: RpcRequest) -> RpcResponse {
    let id = request.id.clone();

    match request.method {
        RpcMethod::Status => handlers::handle_status(state, id).await,
        RpcMethod::Request => handle_llm_request(state, id, request.params).await,
        RpcMethod::Reset => handlers::handle_reset(state, id).await,
        RpcMethod::Uninstall => handlers::handle_uninstall(state, id).await,
        RpcMethod::Autofix => handlers::handle_autofix(state, id).await,
        RpcMethod::Probe => handlers::handle_probe(state, id, request.params).await,
        RpcMethod::Progress => handlers::handle_progress(state, id).await,
        RpcMethod::Stats => handlers::handle_stats(state, id).await,
        RpcMethod::StatusSnapshot => handlers::handle_status_snapshot(state, id).await,
    }
}

/// Service desk pipeline with deterministic routing, triage, and fallback
async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_timeout = { state.read().await.config.daemon.request_timeout_secs };

    // Extract query for timeout fallback (v0.0.40)
    let query_for_fallback = params
        .as_ref()
        .and_then(|p| p.get("prompt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match timeout(Duration::from_secs(request_timeout), handle_llm_request_inner(state.clone(), id.clone(), params, request_id.clone())).await {
        Ok(response) => response,
        Err(_) => {
            warn!("Global request timeout ({}s)", request_timeout);
            make_timeout_response(id, request_id, request_timeout, query_for_fallback.as_deref())
        }
    }
}

fn make_timeout_response(id: String, request_id: String, timeout_secs: u64, query: Option<&str>) -> RpcResponse {
    // v0.0.40: For health queries, use fast path fallback instead of timeout message
    if let Some(q) = query {
        if is_health_query(q) {
            if let Some(fallback) = force_fast_path_fallback(q) {
                info!("Using fast path fallback for health query on timeout");
                let result = build_fast_path_result(
                    request_id,
                    fallback.answer,
                    fallback.class,
                    fallback.reliability,
                    anna_shared::transcript::Transcript::default(),
                );
                return RpcResponse::success(id, serde_json::to_value(result).unwrap());
            }
        }
    }

    // v0.0.34: Never ask to rephrase - provide a deterministic status answer
    let answer = format!(
        "**Request Timeout**\n\n\
         The request exceeded the {}s budget. This typically means:\n\n\
         - The LLM backend is under load or unavailable\n\
         - The query requires complex analysis\n\n\
         **What you can do:**\n\
         - Check `annactl status` to verify LLM availability\n\
         - Try a more specific question (e.g., \"any errors?\" instead of broad queries)\n\n\
         *Evidence: global timeout, no probes completed*",
        timeout_secs
    );

    let result = anna_shared::rpc::ServiceDeskResult {
        request_id, answer, reliability_score: 20, // Low but not zero - we provided info
        reliability_signals: anna_shared::rpc::ReliabilitySignals::default(),
        reliability_explanation: None,
        domain: anna_shared::rpc::SpecialistDomain::System,
        evidence: anna_shared::rpc::EvidenceBlock::default(),
        needs_clarification: false, // Never ask to rephrase
        clarification_question: None,
        transcript: anna_shared::transcript::Transcript::default(),
        execution_trace: Some(anna_shared::trace::ExecutionTrace::global_timeout(timeout_secs)),
    };
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Inner request handler (wrapped by global timeout)
async fn handle_llm_request_inner(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
    request_id: String,
) -> RpcResponse {
    let request_start = Instant::now();
    let mut progress = ProgressTracker::new();

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
    { state.write().await.progress_events.clear(); }

    // Step 0: Fast path check (v0.0.39) - answer health/status queries without LLM
    let fast_path_config = {
        let state = state.read().await;
        (state.config.daemon.fast_path_enabled, state.config.daemon.snapshot_max_age_secs)
    };

    if fast_path_config.0 {
        if let Some(result) = try_fast_path_answer(query, fast_path_config.1) {
            info!("Fast path handled: class={}, reliability={}", result.class, result.reliability);

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
            let fast_result = build_fast_path_result(request_id, result.answer, result.class, result.reliability, progress.take_transcript());
            return RpcResponse::success(id, serde_json::to_value(fast_result).unwrap());
        }
    }

    // Step 1: Deterministic routing (always runs)
    let det_route = router::get_route(query);
    info!("Router: class={:?}, domain={}, probes={:?}", det_route.class, det_route.domain, det_route.probes);

    // Step 2: Route based on query class
    let (mut ticket, triage_result, translator_timed_out) = if det_route.class == router::QueryClass::Unknown {
        // Unknown class -> use triage path with LLM translator
        triage_path(&state, query, &llm_config, &translator_model, hw_cores, hw_ram_gb, has_gpu, &mut progress).await
    } else {
        // Known class -> deterministic ticket
        let ticket = router::apply_deterministic_routing(query, None);
        (ticket, None, false)
    };

    // Step 2.5: Enforce probe spine (v0.45.2 - user text based)
    // FIRST: Use keyword matching on user text to force probes (last line of defense)
    let spine_decision = enforce_minimum_probes(query, &ticket.needs_probes);
    if spine_decision.enforced {
        info!("Probe spine enforced from user text: {}", spine_decision.reason);
        // Convert ProbeId to command strings
        ticket.needs_probes = spine_decision.probes.iter()
            .map(|p| probe_to_command(p))
            .collect();
    } else {
        // FALLBACK: Try route-capability based enforcement
        let (enforced_probes, spine_reason) = enforce_spine_probes(&ticket.needs_probes, &det_route.capability);
        if let Some(ref reason) = spine_reason {
            info!("Probe spine enforced from route: {}", reason);
            ticket.needs_probes = enforced_probes;
        }
    }

    let classified_domain = ticket.domain;
    let ticket_probes_planned = ticket.needs_probes.len();
    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}, confidence={:.2}",
        ticket.domain, ticket.intent, ticket.needs_probes, ticket.confidence
    ));

    // Step 3: Check if immediate clarification needed (from triage)
    if let Some(ref triage) = triage_result {
        if triage.needs_immediate_clarification {
            save_progress(&state, &progress).await;
            let question = triage.clarification_question.clone().unwrap_or_else(|| {
                triage::generate_heuristic_clarification(query)
            });
            let result = service_desk::create_clarification_response(
                request_id, ticket, &question, progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 4: Run probes with timeout
    progress.start_stage(RequestStage::Probes, llm_config.probes_total_timeout_secs);
    let probe_cap_warning = triage_result.as_ref().map(|t| t.probe_cap_applied).unwrap_or(false);
    let probes_start = Instant::now();

    let probe_results = match timeout(
        Duration::from_secs(llm_config.probes_total_timeout_secs),
        probe_runner::run_probes(&state, &ticket, &llm_config, &mut progress),
    ).await {
        Ok(results) => {
            progress.complete_stage(RequestStage::Probes);
            // Record probes latency
            { state.write().await.latency.probes.add(probes_start.elapsed().as_millis() as u64); }
            results
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                request_id, "probes", Some(ticket), vec![], progress.take_transcript(), classified_domain,
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    // Step 5: Build context with summarized probes
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    // Step 6: Try deterministic answer FIRST for known query classes
    let specialist_result = if det_route.can_answer_deterministically() {
        if let Some(det) = deterministic::try_answer(query, &context, &probe_results) {
            if det.parsed_data_count > 0 {
                info!("Deterministic answer produced ({} entries)", det.parsed_data_count);
                // Skip specialist stage - deterministic router answered
                progress.skip_stage_deterministic(RequestStage::Specialist);
                let route_class = det.route_class.clone();
                SpecialistResult {
                    answer: det.answer.clone(),
                    used_deterministic: true,
                    det_result: Some(det),
                    prompt_truncated: false, // No prompt for deterministic path
                    outcome: SpecialistOutcome::Skipped,
                    fallback_route_class: Some(route_class),
                }
            } else {
                warn!("Deterministic parser produced empty result");
                try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, debug_mode, &mut progress).await
            }
        } else {
            try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, debug_mode, &mut progress).await
        }
    } else {
        try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, debug_mode, &mut progress).await
    };
    let SpecialistResult { answer, used_deterministic, det_result, prompt_truncated, outcome, fallback_route_class } = specialist_result;

    // Step 7: Handle no answer case
    if answer.is_empty() {
        save_progress(&state, &progress).await;
        let result = service_desk::create_no_data_response(
            request_id, ticket, probe_results, progress.take_transcript(), classified_domain,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 8: Build final result with proper scoring
    progress.start_stage(RequestStage::Supervisor, llm_config.supervisor_timeout_secs);
    progress.add_final_answer(&answer);

    let parsed_data_count = det_result.as_ref().map(|d| d.parsed_data_count).unwrap_or(0);

    // Build fallback context for TRUST+ explanations and v0.0.24 trace-based scoring
    let used_deterministic_fallback = matches!(outcome, SpecialistOutcome::Timeout | SpecialistOutcome::Error)
        && used_deterministic && parsed_data_count > 0;
    let fallback_used = if used_deterministic_fallback {
        Some(anna_shared::trace::FallbackUsed::Deterministic {
            route_class: fallback_route_class.clone().unwrap_or_default(),
        })
    } else {
        Some(anna_shared::trace::FallbackUsed::None)
    };

    // v0.45.2: Derive evidence kinds from ACTUAL probes, not route class
    let actual_evidence_kinds = evidence_kinds_from_probes(&probe_results);

    let fallback_ctx = service_desk::FallbackContext {
        used_deterministic_fallback,
        fallback_route_class: fallback_route_class.clone().unwrap_or_default(),
        evidence_kinds: actual_evidence_kinds.iter().map(|k| k.to_string()).collect(),
        specialist_outcome: Some(outcome),
        fallback_used,
        // v0.45.x: Pass route capability's evidence_required for proper spine enforcement
        evidence_required: Some(det_route.capability.evidence_required),
    };

    let mut result = service_desk::build_result_with_flags(
        request_id, answer, query, ticket, probe_results.clone(), progress.transcript_clone(),
        classified_domain, translator_timed_out, used_deterministic,
        parsed_data_count, prompt_truncated, fallback_ctx,
    );

    // Step 9: Build execution trace
    let probe_stats = ProbeStats::from_results(ticket_probes_planned, &probe_results);
    let evidence_kinds = actual_evidence_kinds;

    result.execution_trace = Some(match outcome {
        SpecialistOutcome::Skipped => {
            // Deterministic route answered directly
            ExecutionTrace::deterministic_route(
                fallback_route_class.as_deref().unwrap_or("unknown"),
                probe_stats,
                evidence_kinds,
            )
        }
        SpecialistOutcome::Ok => {
            // Specialist LLM answered successfully
            ExecutionTrace::specialist_ok(probe_stats)
        }
        SpecialistOutcome::Timeout => {
            if used_deterministic && parsed_data_count > 0 {
                // Timeout with successful fallback
                ExecutionTrace::specialist_timeout_with_fallback(
                    fallback_route_class.as_deref().unwrap_or("unknown"),
                    probe_stats,
                    evidence_kinds,
                )
            } else {
                // Timeout without successful fallback
                ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
            }
        }
        SpecialistOutcome::Error => {
            if used_deterministic && parsed_data_count > 0 {
                // Error with successful fallback
                ExecutionTrace::specialist_error_with_fallback(
                    fallback_route_class.as_deref().unwrap_or("unknown"),
                    probe_stats,
                    evidence_kinds,
                )
            } else {
                // Error without successful fallback - treat like timeout
                ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
            }
        }
        SpecialistOutcome::BudgetExceeded => {
            // Budget exceeded - similar to timeout
            ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
        }
    });

    // Add probe cap warning to evidence
    if probe_cap_warning {
        result.evidence.last_error = Some("probe_cap_applied".to_string());
    }

    progress.complete_stage(RequestStage::Supervisor);

    // Record total request latency
    let total_ms = request_start.elapsed().as_millis() as u64;
    { state.write().await.latency.total.add(total_ms); }

    info!("Request completed: domain={}, reliability={}, deterministic={}, trace={}, latency={}ms",
          result.domain, result.reliability_score, used_deterministic,
          result.execution_trace.as_ref().map(|t| t.to_string()).unwrap_or_default(), total_ms);

    save_progress(&state, &progress).await;
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Triage path for unknown queries - uses LLM translator with confidence threshold
async fn triage_path(
    state: &SharedState,
    query: &str,
    config: &LlmConfig,
    translator_model: &str,
    hw_cores: u32,
    hw_ram_gb: f64,
    has_gpu: bool,
    progress: &mut ProgressTracker,
) -> (anna_shared::rpc::TranslatorTicket, Option<TriageResult>, bool) {
    progress.start_stage(RequestStage::Translator, config.translator_timeout_secs);
    let translator_input = TranslatorInput::new(query, hw_cores, hw_ram_gb, has_gpu);
    let stage_start = Instant::now();

    let (llm_ticket, translator_timed_out) = match timeout(
        Duration::from_secs(config.translator_timeout_secs),
        translator::translate_with_context(translator_model, &translator_input, config.translator_timeout_secs),
    ).await {
        Ok(Ok(t)) => { progress.complete_stage(RequestStage::Translator); (Some(t), false) }
        Ok(Err(e)) => { warn!("Translator error: {}", e); progress.error_stage(RequestStage::Translator, &e); (None, false) }
        Err(_) => { warn!("Translator timeout"); progress.timeout_stage(RequestStage::Translator); (None, true) }
    };

    // Record translator latency
    { state.write().await.latency.translator.add(stage_start.elapsed().as_millis() as u64); }

    // If translator failed completely, use fallback
    let ticket = llm_ticket.unwrap_or_else(|| triage::create_fallback_ticket(query));

    // Apply triage rules
    let triage_result = triage::apply_triage_rules(ticket.clone());

    (triage_result.ticket.clone(), Some(triage_result), translator_timed_out)
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    state.write().await.progress_events = progress.events().to_vec();
}
