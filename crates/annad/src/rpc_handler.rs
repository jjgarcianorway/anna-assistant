//! RPC request handlers with timeouts and progress tracking.

use anna_shared::progress::{ProgressEvent, RequestStage};
use anna_shared::rpc::{ProbeResult, RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

use crate::config::LlmConfig;
use crate::deterministic;
use crate::handlers;
use crate::ollama;
use crate::probes;
use crate::progress_tracker::ProgressTracker;
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

/// Service desk pipeline with timeouts and deterministic fallback
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
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    let query = &params.prompt;
    progress.add_user_message(query);

    // Clear previous progress events
    {
        let mut state = state.write().await;
        state.progress_events.clear();
    }

    // Step 1: Translator with timeout (fallback on failure)
    // Build minimal translator input (no probe output, just hw summary)
    let translator_input = TranslatorInput::new(query, hw_cores, hw_ram_gb, has_gpu);
    progress.start_stage(RequestStage::Translator, llm_config.translator_timeout_secs);
    let translator_timed_out;

    let ticket = match timeout(
        Duration::from_secs(llm_config.translator_timeout_secs),
        translator::translate_with_context(
            &translator_model,
            &translator_input,
            llm_config.translator_timeout_secs,
        ),
    )
    .await
    {
        Ok(Ok(t)) => {
            progress.complete_stage(RequestStage::Translator);
            translator_timed_out = false;
            t
        }
        Ok(Err(e)) => {
            warn!("Translator error, using fallback: {}", e);
            progress.error_stage(RequestStage::Translator, &e);
            translator_timed_out = false;
            translator::translate_fallback(query)
        }
        Err(_) => {
            warn!("Translator timeout, using fallback");
            progress.timeout_stage(RequestStage::Translator);
            translator_timed_out = true;
            translator::translate_fallback(query)
        }
    };

    // Store the classified domain for consistency
    let classified_domain = ticket.domain;

    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}",
        ticket.domain, ticket.intent, ticket.needs_probes
    ));

    info!(
        "Translator: intent={}, domain={}, confidence={:.2}, probes={:?}",
        ticket.intent, ticket.domain, ticket.confidence, ticket.needs_probes
    );

    // Step 2: Check if clarification needed (only if no probes to run)
    if let Some(question) = ticket.clarification_question.clone() {
        if ticket.needs_probes.is_empty() {
            info!("Clarification needed: {}", question);
            save_progress(&state, &progress).await;
            let result = service_desk::create_clarification_response(
                request_id,
                ticket,
                &question,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 3: Run probes with timeout
    progress.start_stage(RequestStage::Probes, llm_config.probes_total_timeout_secs);

    let probe_results = match timeout(
        Duration::from_secs(llm_config.probes_total_timeout_secs),
        run_probes_with_timeout(&state, &ticket, &llm_config, &mut progress),
    )
    .await
    {
        Ok(results) => {
            progress.complete_stage(RequestStage::Probes);
            results
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(&state, &progress).await;
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
    };

    let successful_probes = probe_results.iter().filter(|p| p.exit_code == 0).count();
    info!(
        "Probes executed: {} ({} successful)",
        probe_results.len(),
        successful_probes
    );

    // Step 4: Build context
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    // Step 5: Try specialist LLM, fall back to deterministic answerer on timeout/error
    progress.start_stage(RequestStage::Specialist, llm_config.specialist_timeout_secs);

    let system_prompt =
        service_desk::build_specialist_prompt(ticket.domain, &context, &probe_results);
    let full_prompt = format!("{}\n\nUser: {}", system_prompt, query);

    let (answer, used_deterministic) = match timeout(
        Duration::from_secs(llm_config.specialist_timeout_secs),
        ollama::chat_with_timeout(&specialist_model, &full_prompt, llm_config.specialist_timeout_secs),
    )
    .await
    {
        Ok(Ok(response)) => {
            progress.complete_stage(RequestStage::Specialist);
            progress.add_specialist_message(&response);
            (response, false)
        }
        Ok(Err(e)) => {
            error!("Specialist LLM error: {}", e);
            progress.error_stage(RequestStage::Specialist, &e.to_string());
            // Try deterministic fallback
            try_deterministic_answer(query, &context, &probe_results, &mut progress)
        }
        Err(_) => {
            warn!("Specialist timeout, trying deterministic fallback");
            progress.timeout_stage(RequestStage::Specialist);
            // Try deterministic fallback
            try_deterministic_answer(query, &context, &probe_results, &mut progress)
        }
    };

    // If no answer at all (deterministic failed and no LLM), ask for clarification
    if answer.is_empty() {
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

    // Step 6: Supervisor - build final result with proper scoring
    progress.start_stage(RequestStage::Supervisor, llm_config.supervisor_timeout_secs);
    progress.add_anna_response(&answer);

    let result = service_desk::build_result_with_flags(
        request_id,
        answer,
        ticket,
        probe_results,
        progress.transcript_clone(),
        classified_domain,
        translator_timed_out,
        used_deterministic,
    );
    progress.complete_stage(RequestStage::Supervisor);

    info!(
        "Request completed: domain={}, reliability={}, probes={}, deterministic={}",
        result.domain,
        result.reliability_score,
        result.evidence.probes_executed.len(),
        used_deterministic
    );

    save_progress(&state, &progress).await;
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Try to produce a deterministic answer from available data
fn try_deterministic_answer(
    query: &str,
    context: &anna_shared::rpc::RuntimeContext,
    probe_results: &[ProbeResult],
    progress: &mut ProgressTracker,
) -> (String, bool) {
    if let Some(answer) = deterministic::try_answer(query, context, probe_results) {
        info!("Deterministic answerer produced response");
        progress.add_specialist_message("[deterministic fallback]");
        (answer, true)
    } else {
        warn!("Deterministic answerer could not produce answer");
        (String::new(), true)
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
            let cached = {
                let state = state.read().await;
                state.get_cached_probe(cmd)
            };

            if let Some(cached_result) = cached {
                info!("Using cached probe result for: {}", cmd);
                let preview = cached_result.stdout.lines().next().map(|s| s.to_string());
                progress.add_probe_end(
                    probe_id,
                    cached_result.exit_code,
                    cached_result.timing_ms,
                    preview,
                );
                progress.add(ProgressEvent::probe_complete(
                    probe_id,
                    cached_result.exit_code,
                    cached_result.timing_ms,
                    progress.elapsed_ms(),
                ));
                results.push(cached_result);
            } else {
                // Run probe with timeout
                let probe_start = Instant::now();
                let result = match timeout(
                    Duration::from_secs(config.probe_timeout_secs),
                    tokio::task::spawn_blocking({
                        let cmd = cmd.to_string();
                        move || probes::run_command_structured(&cmd)
                    }),
                )
                .await
                {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        warn!("Probe task error: {}", e);
                        ProbeResult {
                            command: cmd.to_string(),
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: format!("Task error: {}", e),
                            timing_ms: probe_start.elapsed().as_millis() as u64,
                        }
                    }
                    Err(_) => {
                        warn!("Probe timeout: {}", cmd);
                        ProbeResult {
                            command: cmd.to_string(),
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: "Probe timeout".to_string(),
                            timing_ms: config.probe_timeout_secs * 1000,
                        }
                    }
                };

                let preview = result.stdout.lines().next().map(|s| s.to_string());
                progress.add_probe_end(probe_id, result.exit_code, result.timing_ms, preview);
                progress.add(ProgressEvent::probe_complete(
                    probe_id,
                    result.exit_code,
                    result.timing_ms,
                    progress.elapsed_ms(),
                ));

                // Cache successful results
                if result.exit_code == 0 {
                    let mut state = state.write().await;
                    state.cache_probe(result.clone());
                }

                results.push(result);
            }
        }
    }

    // Clean expired cache
    {
        let mut state = state.write().await;
        state.clean_probe_cache();
    }

    results
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    let mut state = state.write().await;
    state.progress_events = progress.events().to_vec();
}
