//! RPC request handlers with deterministic routing and fallback.

use anna_shared::progress::{ProgressEvent, RequestStage};
use anna_shared::rpc::{ProbeResult, RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

use crate::config::LlmConfig;
use crate::deterministic::{self, DeterministicResult};
use crate::handlers;
use crate::ollama;
use crate::probes;
use crate::progress_tracker::ProgressTracker;
use crate::router;
use crate::service_desk;
use crate::state::SharedState;
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
    }
}

/// Service desk pipeline with deterministic routing and fallback
async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let mut progress = ProgressTracker::new();
    let request_id = uuid::Uuid::new_v4().to_string();

    // Get config, models, and hardware from state
    let (llm_config, translator_model, specialist_model, hw_cores, hw_ram_gb, has_gpu) = {
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

    // Clear previous progress events
    { state.write().await.progress_events.clear(); }

    // Step 1: Deterministic routing (always runs, overrides LLM for known classes)
    let det_route = router::get_route(query);
    info!("Router: class={:?}, domain={}, probes={:?}", det_route.class, det_route.domain, det_route.probes);

    // Step 2: Try LLM translator, then apply deterministic override
    progress.start_stage(RequestStage::Translator, llm_config.translator_timeout_secs);
    let translator_input = TranslatorInput::new(query, hw_cores, hw_ram_gb, has_gpu);
    let translator_timed_out;

    let llm_ticket = match timeout(
        Duration::from_secs(llm_config.translator_timeout_secs),
        translator::translate_with_context(&translator_model, &translator_input, llm_config.translator_timeout_secs),
    ).await {
        Ok(Ok(t)) => { progress.complete_stage(RequestStage::Translator); translator_timed_out = false; Some(t) }
        Ok(Err(e)) => { warn!("Translator error: {}", e); progress.error_stage(RequestStage::Translator, &e); translator_timed_out = false; None }
        Err(_) => { warn!("Translator timeout"); progress.timeout_stage(RequestStage::Translator); translator_timed_out = true; None }
    };

    // Apply deterministic routing (overrides LLM for known classes)
    let ticket = router::apply_deterministic_routing(query, llm_ticket);
    let classified_domain = ticket.domain;

    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}",
        ticket.domain, ticket.intent, ticket.needs_probes
    ));

    // Step 3: Check if clarification needed (only for unknown classes with no probes)
    if let Some(question) = ticket.clarification_question.clone() {
        if ticket.needs_probes.is_empty() && det_route.class == router::QueryClass::Unknown {
            save_progress(&state, &progress).await;
            let result = service_desk::create_clarification_response(
                request_id, ticket, &question, progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 4: Run probes with timeout
    progress.start_stage(RequestStage::Probes, llm_config.probes_total_timeout_secs);
    let probe_results = match timeout(
        Duration::from_secs(llm_config.probes_total_timeout_secs),
        run_probes_with_timeout(&state, &ticket, &llm_config, &mut progress),
    ).await {
        Ok(results) => { progress.complete_stage(RequestStage::Probes); results }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                request_id, "probes", Some(ticket), vec![], progress.take_transcript(), classified_domain,
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    // Step 5: Build context
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    // Step 6: Try deterministic answer FIRST for known query classes
    let (answer, used_deterministic, det_result) = if det_route.can_answer_deterministically {
        if let Some(det) = deterministic::try_answer(query, &context, &probe_results) {
            if det.parsed_data_count > 0 {
                info!("Deterministic answer produced ({} entries)", det.parsed_data_count);
                progress.start_stage(RequestStage::Specialist, 0);
                progress.complete_stage(RequestStage::Specialist);
                progress.add_specialist_message("[deterministic]");
                (det.answer.clone(), true, Some(det))
            } else {
                // Parser produced empty result - try LLM
                warn!("Deterministic parser produced empty result");
                try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, &mut progress).await
            }
        } else {
            // No deterministic answer available - try LLM
            try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, &mut progress).await
        }
    } else {
        // Unknown query class - use LLM specialist
        try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, &mut progress).await
    };

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
    progress.add_anna_response(&answer);

    let result = service_desk::build_result_with_flags(
        request_id, answer, ticket, probe_results, progress.transcript_clone(),
        classified_domain, translator_timed_out, used_deterministic,
        det_result.map(|d| d.parsed_data_count).unwrap_or(0),
    );
    progress.complete_stage(RequestStage::Supervisor);

    info!("Request completed: domain={}, reliability={}, deterministic={}",
          result.domain, result.reliability_score, used_deterministic);

    save_progress(&state, &progress).await;
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Try specialist LLM, return (answer, used_deterministic, det_result)
async fn try_specialist_llm(
    state: &SharedState,
    query: &str,
    context: &anna_shared::rpc::RuntimeContext,
    probe_results: &[ProbeResult],
    ticket: &anna_shared::rpc::TranslatorTicket,
    config: &LlmConfig,
    model: &str,
    progress: &mut ProgressTracker,
) -> (String, bool, Option<DeterministicResult>) {
    progress.start_stage(RequestStage::Specialist, config.specialist_timeout_secs);

    let system_prompt = service_desk::build_specialist_prompt(ticket.domain, context, probe_results);
    let full_prompt = format!("{}\n\nUser: {}", system_prompt, query);

    match timeout(
        Duration::from_secs(config.specialist_timeout_secs),
        ollama::chat_with_timeout(model, &full_prompt, config.specialist_timeout_secs),
    ).await {
        Ok(Ok(response)) => {
            progress.complete_stage(RequestStage::Specialist);
            progress.add_specialist_message(&response);
            (response, false, None)
        }
        Ok(Err(e)) => {
            error!("Specialist LLM error: {}", e);
            progress.error_stage(RequestStage::Specialist, &e.to_string());
            try_deterministic_fallback(query, context, probe_results, progress)
        }
        Err(_) => {
            warn!("Specialist timeout, trying deterministic fallback");
            progress.timeout_stage(RequestStage::Specialist);
            try_deterministic_fallback(query, context, probe_results, progress)
        }
    }
}

/// Try deterministic fallback after LLM failure
fn try_deterministic_fallback(
    query: &str,
    context: &anna_shared::rpc::RuntimeContext,
    probe_results: &[ProbeResult],
    progress: &mut ProgressTracker,
) -> (String, bool, Option<DeterministicResult>) {
    if let Some(det) = deterministic::try_answer(query, context, probe_results) {
        if det.parsed_data_count > 0 {
            info!("Deterministic fallback produced answer");
            progress.add_specialist_message("[deterministic fallback]");
            (det.answer.clone(), true, Some(det))
        } else {
            warn!("Deterministic fallback produced empty result");
            (String::new(), true, Some(det))
        }
    } else {
        warn!("Deterministic fallback could not produce answer");
        (String::new(), true, None)
    }
}

/// Run probes with individual timeouts
async fn run_probes_with_timeout(
    state: &SharedState,
    ticket: &anna_shared::rpc::TranslatorTicket,
    config: &LlmConfig,
    progress: &mut ProgressTracker,
) -> Vec<ProbeResult> {
    let mut results = Vec::new();

    for probe_id in &ticket.needs_probes {
        if let Some(cmd) = translator::probe_id_to_command(probe_id) {
            progress.add(ProgressEvent::probe_running(probe_id, progress.elapsed_ms()));
            progress.add_probe_start(probe_id, cmd);

            // Check cache first
            let cached = { state.read().await.get_cached_probe(cmd) };

            if let Some(cached_result) = cached {
                info!("Using cached probe: {}", cmd);
                let preview = cached_result.stdout.lines().next().map(|s| s.to_string());
                progress.add_probe_end(probe_id, cached_result.exit_code, cached_result.timing_ms, preview);
                progress.add(ProgressEvent::probe_complete(probe_id, cached_result.exit_code, cached_result.timing_ms, progress.elapsed_ms()));
                results.push(cached_result);
            } else {
                let probe_start = Instant::now();
                let result = match timeout(
                    Duration::from_secs(config.probe_timeout_secs),
                    tokio::task::spawn_blocking({ let cmd = cmd.to_string(); move || probes::run_command_structured(&cmd) }),
                ).await {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => { warn!("Probe task error: {}", e); ProbeResult { command: cmd.to_string(), exit_code: -1, stdout: String::new(), stderr: format!("Task error: {}", e), timing_ms: probe_start.elapsed().as_millis() as u64 } }
                    Err(_) => { warn!("Probe timeout: {}", cmd); ProbeResult { command: cmd.to_string(), exit_code: -1, stdout: String::new(), stderr: "Probe timeout".to_string(), timing_ms: config.probe_timeout_secs * 1000 } }
                };

                let preview = result.stdout.lines().next().map(|s| s.to_string());
                progress.add_probe_end(probe_id, result.exit_code, result.timing_ms, preview);
                progress.add(ProgressEvent::probe_complete(probe_id, result.exit_code, result.timing_ms, progress.elapsed_ms()));

                if result.exit_code == 0 {
                    state.write().await.cache_probe(result.clone());
                }
                results.push(result);
            }
        }
    }

    { state.write().await.clean_probe_cache(); }
    results
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    state.write().await.progress_events = progress.events().to_vec();
}
