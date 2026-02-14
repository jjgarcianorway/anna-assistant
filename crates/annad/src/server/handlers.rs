//! RPC request handlers and connection management.
//! v0.0.922: Added request deduplication
//! v0.0.926: Added memory fast path
//! v0.3.21: Updated reset to use SafeReset with modes and backups

use anna_shared::rpc::{ResetMode, ResetParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::safe_ops::SafeReset;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{info, warn};

use crate::core_loop::cache::{
    get_cached_answer, is_request_inflight, register_inflight_request, complete_inflight_request,
    check_memory_fast_path, boost_experience_usefulness,
};
use crate::core_loop::execute_question;
use crate::state::SharedState;

use super::streaming::handle_streaming_request;

/// Handle a single client connection
pub async fn handle_connection(stream: UnixStream, state: SharedState) -> Result<()> {
    // v0.3.171: Refresh system identity to detect current user (daemon may have started before user logged in)
    crate::system_identity::refresh_system_identity();

    // Track active connections for graceful shutdown
    {
        let mut state_guard = state.write().await;
        state_guard.connection_started();
    }

    // Ensure we decrement the counter when done
    let _guard = ConnectionGuard {
        state: state.clone(),
    };

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    reader.read_line(&mut line).await?;

    let request: RpcRequest = match serde_json::from_str(&line) {
        Ok(r) => r,
        Err(e) => {
            let response = RpcResponse::error("", -32700, &format!("Parse error: {}", e));
            let response_json = serde_json::to_string(&response)?;
            writer
                .write_all(format!("{}\n", response_json).as_bytes())
                .await?;
            return Ok(());
        }
    };

    // Handle streaming requests separately
    if matches!(request.method, RpcMethod::AskStreaming) {
        return handle_streaming_request(request, state, writer).await;
    }

    let response = handle_request(request, state).await;
    let response_json = serde_json::to_string(&response)?;
    writer
        .write_all(format!("{}\n", response_json).as_bytes())
        .await?;

    Ok(())
}

/// Handle an RPC request
pub async fn handle_request(request: RpcRequest, state: SharedState) -> RpcResponse {
    match request.method {
        RpcMethod::Status => {
            let state = state.read().await;
            let status = state.to_status();
            match serde_json::to_value(&status) {
                Ok(result) => RpcResponse::success(&request.id, result),
                Err(e) => {
                    RpcResponse::error(&request.id, -32603, &format!("Internal error: {}", e))
                }
            }
        }
        RpcMethod::Ask => {
            // Extract the question from params
            let question = request
                .params
                .as_ref()
                .and_then(|p| p.get("question"))
                .and_then(|q| q.as_str())
                .unwrap_or("");

            if question.is_empty() {
                return RpcResponse::error(&request.id, -32602, "Missing 'question' parameter");
            }

            // v0.0.926: Check memory for high-confidence matches (fast path)
            if let Some(memory_result) = check_memory_fast_path(question) {
                info!("Memory fast path: returning learned answer (confidence={:.2})", memory_result.confidence);
                // Boost the experience usefulness in background
                let exp_id = memory_result.experience_id.clone();
                tokio::spawn(async move {
                    boost_experience_usefulness(&exp_id);
                });
                let result = anna_shared::rpc::AskResult {
                    answer: memory_result.answer,
                    success: true,
                    iterations: 0,
                    commands_executed: memory_result.commands,
                    dialogue: vec![],
                    needs_clarification: false,
                    clarification_question: None,
                    cached: true, // Mark as cached since it's from memory
                    citations: vec![],
                    abstained: false,
                    final_confidence: None,
                };
                return match serde_json::to_value(&result) {
                    Ok(v) => RpcResponse::success(&request.id, v),
                    Err(e) => RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e)),
                };
            }

            // v0.0.922: Check if this exact question is already being processed
            if is_request_inflight(question) {
                info!("Request deduplication: waiting for in-flight request");
                // Wait a bit and check cache for result
                for _ in 0..30 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    if let Some((cached_answer, _)) = get_cached_answer(question) {
                        info!("Request deduplication: returning cached result");
                        let result = anna_shared::rpc::AskResult {
                            answer: cached_answer,
                            success: true,
                            iterations: 0,
                            commands_executed: vec![],
                            dialogue: vec![],
                            needs_clarification: false,
                            clarification_question: None,
                            cached: true,
                            citations: vec![],
                            abstained: false,
                            final_confidence: None,
                        };
                        return match serde_json::to_value(&result) {
                            Ok(v) => RpcResponse::success(&request.id, v),
                            Err(e) => RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e)),
                        };
                    }
                    if !is_request_inflight(question) {
                        break;
                    }
                }
            }

            // Get model from state
            let model = {
                let state = state.read().await;
                match &state.model {
                    Some(m) => m.clone(),
                    None => {
                        // Use init_status directly — it's updated at every healing step
                        // (installing ollama, starting service, downloading model, etc.)
                        // with step numbers, ETAs, and reason. Fall back to last_error if set.
                        let msg = if let Some(ref err) = state.last_error {
                            format!("Anna is recovering (retrying automatically): {}", err)
                        } else if !state.init_status.is_empty() && state.init_status != "Ready" {
                            state.init_status.clone()
                        } else {
                            "Anna is initializing — please try again in a moment.".to_string()
                        };
                        return RpcResponse::error(&request.id, -32603, &msg);
                    }
                }
            };

            // v0.0.922: Register this request as in-flight
            register_inflight_request(question);

            // Execute the question
            let result = match execute_question(&model, question).await {
                Ok(result) => match serde_json::to_value(&result) {
                    Ok(v) => RpcResponse::success(&request.id, v),
                    Err(e) => {
                        RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                    }
                },
                Err(e) => {
                    // v0.0.927: Graceful degradation with helpful error messages
                    let err_str = e.to_string().to_lowercase();
                    let is_infra_failure = err_str.contains("circuit breaker")
                        || err_str.contains("connection")
                        || err_str.contains("refused")
                        || err_str.contains("404")
                        || err_str.contains("model") && err_str.contains("not found");

                    // Reset state so the init loop re-triggers reinstall/recovery
                    if is_infra_failure {
                        let s = state.clone();
                        tokio::spawn(async move {
                            let mut guard = s.write().await;
                            guard.model = None;
                            guard.state = anna_shared::status::DaemonState::Starting;
                            guard.init_status = "Ollama unavailable — recovering automatically...".to_string();
                        });
                    }

                    let user_message = if err_str.contains("circuit breaker") {
                        "Ollama is temporarily unavailable due to repeated failures. \
                         Anna is attempting recovery automatically — please try again in a moment."
                    } else if err_str.contains("timeout") {
                        "The request timed out. The model may still be loading — \
                         please try again in a few seconds."
                    } else if err_str.contains("connection") || err_str.contains("refused") {
                        "Ollama is not running. Anna is reinstalling and restarting it automatically — \
                         please try again in a moment."
                    } else if err_str.contains("404") {
                        "The model is not available. Anna is recovering automatically — \
                         please try again in a moment."
                    } else if err_str.contains("model") && err_str.contains("not found") {
                        "The configured model is not available. \
                         Anna is attempting to recover automatically."
                    } else {
                        // Return original error for unknown cases — complete registration first
                        complete_inflight_request(question);
                        return RpcResponse::error(&request.id, -32603, &format!("Execution error: {}", e));
                    };

                    // Return a helpful response instead of cryptic error
                    let result = anna_shared::rpc::AskResult {
                        answer: user_message.to_string(),
                        success: false,
                        iterations: 0,
                        commands_executed: vec![],
                        dialogue: vec![],
                        needs_clarification: false,
                        clarification_question: None,
                        cached: false,
                        citations: vec![],
                        abstained: false,
                        final_confidence: None,
                    };
                    match serde_json::to_value(&result) {
                        Ok(v) => RpcResponse::success(&request.id, v),
                        Err(e) => RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e)),
                    }
                }
            };

            // v0.0.922: Mark request as complete
            complete_inflight_request(question);

            result
        }
        RpcMethod::AskStreaming => {
            // This is handled separately in handle_streaming_request
            // Should not reach here, but provide a fallback
            RpcResponse::error(
                &request.id,
                -32603,
                "Use streaming connection for AskStreaming",
            )
        }
        RpcMethod::Reset => {
            // v0.3.21: Parse reset params (supports modes and backups)
            let params: ResetParams = request.params
                .as_ref()
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();

            info!("Processing reset request with mode: {:?}", params.mode);

            // v0.3.28: Clear ALL in-memory state atomically to ensure consistency
            // Order matters: clear global caches first, then state, then files

            // 1. Clear global in-memory caches (command cache, answer cache, LLM memo, etc.)
            crate::core_loop::cache::clear_all_caches();

            // 2. Clear daemon state (answer cache, sessions)
            {
                let mut state_guard = state.write().await;
                state_guard.clear_for_reset();
            }

            // 3. Clear in-memory ticket store
            crate::department::tickets::reset_ticket_store();

            // Use SafeReset for file-based resets (with backup)
            match SafeReset::execute(params.mode) {
                Ok(mut result) => {
                    // Add in-memory items to cleared list
                    result.cleared.insert(0, "In-memory caches".to_string());
                    result.cleared.insert(1, "Sessions".to_string());

                    info!("Reset complete: {:?}", result.cleared);
                    if let Some(ref backup) = result.backup_path {
                        info!("Backup created at: {}", backup);
                    }

                    match serde_json::to_value(&result) {
                        Ok(v) => RpcResponse::success(&request.id, v),
                        Err(e) => {
                            RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                        }
                    }
                }
                Err(e) => {
                    warn!("Reset failed: {}", e);
                    RpcResponse::error(&request.id, -32603, &format!("Reset failed: {}", e))
                }
            }
        }
        RpcMethod::DiagnoseWifi => {
            // Phase 43: WiFi diagnosis handler
            info!("Processing WiFi diagnosis request");

            use crate::assisted_ops::wifi_diagnosis::diagnose_slow_wifi;

            match diagnose_slow_wifi() {
                Some(operation) => {
                    // Convert internal types to RPC types
                    let result = convert_assisted_op_to_rpc(&operation);
                    match serde_json::to_value(&result) {
                        Ok(v) => RpcResponse::success(&request.id, v),
                        Err(e) => {
                            RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                        }
                    }
                }
                None => {
                    // No WiFi issue detected - return helpful message
                    info!("WiFi diagnosis found no issues");
                    RpcResponse::error(
                        &request.id,
                        -32603,
                        "No WiFi issues detected. Your WiFi appears to be working correctly.",
                    )
                }
            }
        }
        RpcMethod::GenerateReport => {
            // v0.3.159: Direct PDF generation handler
            info!("Processing PDF report generation request");

            match crate::report::generate_pdf_report() {
                Ok(path) => {
                    info!("PDF report generated at: {}", path.display());
                    let path_str = path.to_string_lossy().to_string();
                    match serde_json::to_value(&path_str) {
                        Ok(v) => RpcResponse::success(&request.id, v),
                        Err(e) => {
                            RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                        }
                    }
                }
                Err(e) => {
                    warn!("PDF report generation failed: {}", e);
                    RpcResponse::error(&request.id, -32603, &format!("Report generation failed: {}", e))
                }
            }
        }
        RpcMethod::SendReportToTelegram => {
            // v0.3.159: Send PDF report to Telegram
            info!("Processing send report to Telegram request");

            let path_str = request
                .params
                .as_ref()
                .and_then(|p| p.get("path"))
                .and_then(|p| p.as_str())
                .ok_or_else(|| "Missing 'path' parameter");

            match path_str {
                Ok(path) => {
                    let path_buf = std::path::PathBuf::from(path);
                    if !path_buf.exists() {
                        return RpcResponse::error(&request.id, -32602, "Report file not found");
                    }

                    crate::telegram::notifier::send_pdf_report(&path_buf);
                    info!("PDF report queued for Telegram delivery");

                    match serde_json::to_value(&true) {
                        Ok(v) => RpcResponse::success(&request.id, v),
                        Err(e) => {
                            RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                        }
                    }
                }
                Err(e) => {
                    RpcResponse::error(&request.id, -32602, e)
                }
            }
        }
    }
}

/// Convert internal AssistedOperation to RPC wire format (Phase 43).
fn convert_assisted_op_to_rpc(
    op: &crate::assisted_ops::AssistedOperation,
) -> anna_shared::rpc::AssistedOperationResult {
    use anna_shared::rpc::{
        AssistedOperationResult, CommandSafety as RpcCommandSafety, ProposedStepResult,
        RiskLevel as RpcRiskLevel, SourceResult, SourceType as RpcSourceType,
    };
    use crate::assisted_ops::{CommandSafety, RiskLevel, SourceType};

    AssistedOperationResult {
        operation_id: op.operation_id.clone(),
        detected_problem: op.detected_problem.clone(),
        explanation: op.explanation.clone(),
        proposed_steps: op
            .proposed_steps
            .iter()
            .map(|s| ProposedStepResult {
                step_number: s.step_number,
                description: s.description.clone(),
                exact_command: s.exact_command.clone(),
                why: s.why.clone(),
                reversible: s.reversible,
                reverse_command: s.reverse_command.clone(),
                safety: match s.safety {
                    CommandSafety::SafeAutomatic => RpcCommandSafety::SafeAutomatic,
                    CommandSafety::ManualOnly => RpcCommandSafety::ManualOnly,
                },
            })
            .collect(),
        risk_level: match op.risk_level {
            RiskLevel::Low => RpcRiskLevel::Low,
            RiskLevel::Medium => RpcRiskLevel::Medium,
            RiskLevel::High => RpcRiskLevel::High,
            RiskLevel::Critical => RpcRiskLevel::Critical,
        },
        sources: op
            .sources
            .iter()
            .map(|s| SourceResult {
                source_type: match s.source_type {
                    SourceType::ArchWiki => RpcSourceType::ArchWiki,
                    SourceType::ManPage => RpcSourceType::ManPage,
                    SourceType::Upstream => RpcSourceType::Upstream,
                    SourceType::Kernel => RpcSourceType::Kernel,
                    SourceType::Community => RpcSourceType::Community,
                },
                title: s.title.clone(),
                reference: s.reference.clone(),
            })
            .collect(),
        requires_reboot: op.requires_reboot,
        diagnosis_summary: op.diagnosis_summary.clone(),
    }
}

/// RAII guard to track active connections
/// Decrements the counter when dropped (even on error/panic)
struct ConnectionGuard {
    state: SharedState,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        // Use blocking lock since we're in Drop
        // This is safe because we're not in an async context at drop time
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut state_guard = state.write().await;
            state_guard.connection_ended();
        });
    }
}
