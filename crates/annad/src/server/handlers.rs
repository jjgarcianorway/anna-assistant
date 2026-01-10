//! RPC request handlers and connection management.

use anna_shared::rpc::{RpcMethod, RpcRequest, RpcResponse};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::warn;

use crate::core_loop::execute_question;
use crate::state::SharedState;

use super::streaming::handle_streaming_request;

/// Handle a single client connection
pub async fn handle_connection(stream: UnixStream, state: SharedState) -> Result<()> {
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

            // Get model from state
            let model = {
                let state = state.read().await;
                match &state.model {
                    Some(m) => m.clone(),
                    None => {
                        return RpcResponse::error(
                            &request.id,
                            -32603,
                            "Daemon not ready - no model available",
                        );
                    }
                }
            };

            // Execute the question
            match execute_question(&model, question).await {
                Ok(result) => match serde_json::to_value(&result) {
                    Ok(v) => RpcResponse::success(&request.id, v),
                    Err(e) => {
                        RpcResponse::error(&request.id, -32603, &format!("Serialize error: {}", e))
                    }
                },
                Err(e) => {
                    RpcResponse::error(&request.id, -32603, &format!("Execution error: {}", e))
                }
            }
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
