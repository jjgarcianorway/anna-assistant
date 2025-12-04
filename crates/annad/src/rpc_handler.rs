//! RPC request handlers with timeouts and progress tracking.

use anna_shared::progress::{ProgressEvent, RequestStage, TimeoutConfig};
use anna_shared::rpc::{ProbeResult, RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

use crate::handlers;
use crate::ollama;
use crate::probes;
use crate::progress_tracker::ProgressTracker;
use crate::service_desk;
use crate::state::SharedState;
use crate::translator;

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

/// Service desk pipeline with timeouts: translate -> dispatch -> specialist -> supervisor
async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let config = TimeoutConfig::default();
    let mut progress = ProgressTracker::new();
    let request_id = uuid::Uuid::new_v4().to_string();

    // Check if LLM is ready and get model name
    let model = {
        let state = state.read().await;
        if state.llm.state != LlmState::Ready {
            return RpcResponse::error(id, -32002, format!("LLM not ready: {}", state.llm.state));
        }
        state
            .llm
            .models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "llama3.2:1b".to_string())
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

    // Step 1: Translator with timeout
    progress.start_stage(RequestStage::Translator, config.translator_secs);

    let ticket = match timeout(
        Duration::from_secs(config.translator_secs),
        translator::translate(&model, query),
    )
    .await
    {
        Ok(Ok(t)) => {
            progress.complete_stage(RequestStage::Translator);
            t
        }
        Ok(Err(e)) => {
            warn!("Translator error, using fallback: {}", e);
            progress.error_stage(RequestStage::Translator, &e);
            translator::translate_fallback(query)
        }
        Err(_) => {
            warn!("Translator timeout, using fallback");
            progress.timeout_stage(RequestStage::Translator);
            translator::translate_fallback(query)
        }
    };

    // Add translator output to transcript
    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}",
        ticket.domain, ticket.intent, ticket.needs_probes
    ));

    info!(
        "Translator: intent={}, domain={}, confidence={:.2}, probes={:?}",
        ticket.intent, ticket.domain, ticket.confidence, ticket.needs_probes
    );

    // Step 2: Check if clarification needed
    if let Some(question) = ticket.clarification_question.clone() {
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

    // Step 3: Run probes with timeout
    progress.start_stage(RequestStage::Probes, config.probes_total_secs);

    let probe_results = match timeout(
        Duration::from_secs(config.probes_total_secs),
        run_probes_with_timeout(&state, &ticket, &config, &mut progress),
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
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    info!(
        "Probes executed: {} ({} successful)",
        probe_results.len(),
        probe_results.iter().filter(|p| p.exit_code == 0).count()
    );

    // Step 4: Build context and specialist prompt
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    let system_prompt =
        service_desk::build_specialist_prompt(ticket.domain, &context, &probe_results);
    let full_prompt = format!("{}\n\nUser: {}", system_prompt, query);

    // Step 5: Specialist with timeout
    progress.start_stage(RequestStage::Specialist, config.specialist_secs);

    let answer = match timeout(
        Duration::from_secs(config.specialist_secs),
        ollama::chat_with_timeout(&model, &full_prompt, config.specialist_secs),
    )
    .await
    {
        Ok(Ok(response)) => {
            progress.complete_stage(RequestStage::Specialist);
            progress.add_specialist_message(&response);
            response
        }
        Ok(Err(e)) => {
            error!("Specialist LLM error: {}", e);
            progress.error_stage(RequestStage::Specialist, &e.to_string());
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                request_id,
                "specialist",
                Some(ticket),
                probe_results,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Specialist);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                request_id,
                "specialist",
                Some(ticket),
                probe_results,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    // Step 6: Supervisor
    progress.start_stage(RequestStage::Supervisor, config.supervisor_secs);
    progress.add_anna_response(&answer);
    let result = service_desk::build_result(
        request_id,
        answer,
        ticket,
        probe_results,
        progress.transcript_clone(),
    );
    progress.complete_stage(RequestStage::Supervisor);

    info!(
        "Request completed: domain={}, reliability={}, probes={}",
        result.domain,
        result.reliability_score,
        result.evidence.probes_executed.len()
    );

    save_progress(&state, &progress).await;
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Run probes with individual timeouts
async fn run_probes_with_timeout(
    state: &SharedState,
    ticket: &anna_shared::rpc::TranslatorTicket,
    config: &TimeoutConfig,
    progress: &mut ProgressTracker,
) -> Vec<ProbeResult> {
    let mut results = Vec::new();

    for probe_id in &ticket.needs_probes {
        if let Some(cmd) = translator::probe_id_to_command(probe_id) {
            progress.add(ProgressEvent::probe_running(
                probe_id,
                progress.elapsed_ms(),
            ));
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
                    Duration::from_secs(config.probe_each_secs),
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
                            timing_ms: config.probe_each_secs * 1000,
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
