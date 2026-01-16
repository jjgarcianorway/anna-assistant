//! Streaming request handlers.
//! v0.0.998: Added Hollywood IT teams experience
//! v0.3.49: Phase 16 - Action plan execution

use anna_shared::rpc::{DialogueStep, RpcRequest, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::autofix::{get_fix_history_summary, take_pending_autofix};
use crate::plan_executor::{has_pending_plan, is_plan_expired, take_pending_plan};
use crate::plan_generator;
use crate::recipes;
use crate::state::SharedState;

use crate::server::alerts::get_pending_alerts;
use super::confirm_handlers::{
    handle_expired_plan, handle_pending_autofix, handle_pending_plan, handle_pending_recipe,
    handle_recipe_match, handle_template_plan,
};
use super::helpers::{is_fix_history_question, send_filtered_final_answer, take_pending_recipe};

/// Handle a streaming AskStreaming request
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

    // Extract session_id from params (client generates it)
    let session_id = request
        .params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(|s| s.as_str())
        .unwrap_or("default");

    // v0.2.8: Track response time for RPG stats
    let start_time = std::time::Instant::now();

    if question.is_empty() {
        let response = StreamingResponse::Error {
            message: "Missing 'question' parameter".to_string(),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // Phase 16: Check if this is a response to a pending action plan
    // First check if a plan exists but has expired
    if has_pending_plan(session_id) && is_plan_expired(session_id) {
        // Take and discard the expired plan, then notify user
        let _ = take_pending_plan(session_id);
        return handle_expired_plan(session_id, &mut writer).await;
    }
    if let Some(pending_plan) = take_pending_plan(session_id) {
        return handle_pending_plan(pending_plan, question, session_id, &mut writer).await;
    }

    // v0.0.994: Check if this is a response to a pending autofix
    if let Some(pending_fix) = take_pending_autofix(session_id) {
        return handle_pending_autofix(pending_fix, question, session_id, &mut writer).await;
    }

    // v0.0.997: Check if user is asking about fix history
    if is_fix_history_question(question) {
        return handle_fix_history_question(&mut writer).await;
    }

    // v0.0.998: Check if this is a response to a pending recipe
    if let Some(pending_recipe_id) = take_pending_recipe(session_id) {
        return handle_pending_recipe(pending_recipe_id, question, session_id, &mut writer).await;
    }

    // v0.0.998: Check if this matches a configuration recipe
    if let Some(recipe_result) = recipes::try_recipe(question) {
        return handle_recipe_match(recipe_result, question, session_id, &mut writer).await;
    }

    // Phase 16: Check if this matches an action plan template
    if let Some(plan) = plan_generator::generate_template_plan(question) {
        // NOOP short-circuit: if preflight determined no changes needed,
        // emit terminal response without entering confirmation flow.
        // This prevents "Proceed?" prompts when action set is empty.
        if !plan.changes_needed {
            let msg = format!(
                "No changes needed. {}",
                plan.skip_reason.as_deref().unwrap_or("Already configured.")
            );
            send_filtered_final_answer(&mut writer, &msg).await?;

            let result = anna_shared::rpc::AskResult {
                answer: msg,
                success: true,
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
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
        return handle_template_plan(plan, session_id, &mut writer).await;
    }

    // Check for pending critical system alerts and notify user
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

    // Main question processing
    super::main_handler::handle_main_question(question, session_id, state, start_time, &mut writer).await
}

/// Handle fix history question
async fn handle_fix_history_question(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("User asking about fix history");
    let summary = get_fix_history_summary();

    // Phase 15: Filter through ExposureGate
    send_filtered_final_answer(writer, &summary).await?;

    let result = anna_shared::rpc::AskResult {
        answer: summary,
        success: true,
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
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}
