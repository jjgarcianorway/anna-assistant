//! Streaming request handlers.
//! LLM-first architecture: all requests go to the LLM.
//! Only stateful confirmations (pending plans/autofixes) are intercepted.

use anna_shared::rpc::{DialogueStep, RpcRequest, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::autofix::take_pending_autofix;
use crate::plan_executor::{has_pending_plan, is_plan_expired, take_pending_plan};
use crate::state::SharedState;

use crate::server::alerts::get_pending_alerts;
use super::confirm_handlers::{
    handle_expired_plan, handle_pending_autofix, handle_pending_plan,
};

/// Handle a streaming AskStreaming request.
/// LLM-first: no capability routing, no pattern matching, no templates.
/// The LLM reasons about every request.
pub async fn handle_streaming_request(
    request: RpcRequest,
    state: SharedState,
    mut writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let question = request
        .params
        .as_ref()
        .and_then(|p| p.get("question"))
        .and_then(|q| q.as_str())
        .unwrap_or("");

    let session_id = request
        .params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(|s| s.as_str())
        .unwrap_or("default");

    let start_time = std::time::Instant::now();

    if question.is_empty() {
        let response = StreamingResponse::Error {
            message: "Missing 'question' parameter".to_string(),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // Stateful: Check if this is a response to a pending action plan
    if has_pending_plan(session_id) && is_plan_expired(session_id) {
        let _ = take_pending_plan(session_id);
        return handle_expired_plan(session_id, &mut writer).await;
    }
    if let Some(pending_plan) = take_pending_plan(session_id) {
        return handle_pending_plan(pending_plan, question, session_id, &mut writer).await;
    }

    // Stateful: Check if this is a response to a pending autofix
    if let Some(pending_fix) = take_pending_autofix(session_id) {
        return handle_pending_autofix(pending_fix, question, session_id, &mut writer).await;
    }

    // Check for pending critical system alerts
    if let Some(alerts) = get_pending_alerts() {
        for alert in alerts {
            let step = DialogueStep {
                step_type: StepType::SystemAlert,
                content: alert,
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
        }
    }

    // LLM-first: all questions go to the main handler (Ralph loop)
    info!("LLM-first: routing to main handler: {}", question);
    super::main_handler::handle_main_question(question, session_id, state, start_time, &mut writer).await
}
