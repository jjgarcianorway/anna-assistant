//! Streaming request handling for real-time responses.

use anna_shared::rpc::{DialogueStep, RpcRequest, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use crate::core_loop::execute_question_streaming;
use crate::state::SharedState;

use super::alerts::get_pending_alerts;

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

    if question.is_empty() {
        let response = StreamingResponse::Error {
            message: "Missing 'question' parameter".to_string(),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
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

    // Check cache for identical recent question
    {
        let state_guard = state.read().await;
        if let Some(cached_answer) = state_guard.get_cached_answer(question) {
            info!("Returning cached answer for: {}", question);
            // Send cached answer as a quick streaming response
            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: cached_answer.clone(),
            };
            let response = StreamingResponse::Step { step: step.clone() };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // Send done with AskResult
            let result = anna_shared::rpc::AskResult {
                answer: cached_answer,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![step],
                needs_clarification: false,
                clarification_question: None,
                cached: true,
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
    }

    // Get session context and expand question with references
    let (expanded_question, session_context) = {
        let mut state_guard = state.write().await;
        let session = state_guard.get_or_create_session(session_id);
        let expanded = session.expand_question(question);
        let context = if session.history.is_empty() {
            None
        } else {
            Some(session.get_context_for_llm())
        };
        (expanded, context)
    };

    // Get model from state
    let model = {
        let state_guard = state.read().await;
        match &state_guard.model {
            Some(m) => m.clone(),
            None => {
                let response = StreamingResponse::Error {
                    message: "Daemon not ready - no model available".to_string(),
                };
                let json = serde_json::to_string(&response)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                return Ok(());
            }
        }
    };

    // Execute with streaming (use expanded question if different)
    let question_to_use = if expanded_question != question {
        info!(
            "Expanded question with session context: {} -> {}",
            question, expanded_question
        );
        &expanded_question
    } else {
        question
    };

    // v0.0.905: Check answer cache before running LLM
    {
        let state_guard = state.read().await;
        if let Some(cached_answer) = state_guard.get_cached_answer(question_to_use) {
            info!("Returning cached answer for: {}", question_to_use);

            // Send cached response with dialogue showing it's cached
            let step = DialogueStep {
                step_type: StepType::UserQuestion,
                content: question_to_use.to_string(),
            };
            let json = serde_json::to_string(&StreamingResponse::Step { step: step.clone() })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: cached_answer.clone(),
            };
            let json = serde_json::to_string(&StreamingResponse::Step { step: step.clone() })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let result = anna_shared::rpc::AskResult {
                answer: cached_answer,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: true,
            };
            let json = serde_json::to_string(&StreamingResponse::Done { result })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
    }

    let result = execute_question_streaming(
        &model,
        question_to_use,
        session_context.as_deref(),
        &mut writer,
    )
    .await;

    // v0.0.892: Record full turn to session after execution
    match &result {
        Ok(ask_result) => {
            let mut state_guard = state.write().await;
            if let Some(session) = state_guard.sessions.sessions.get_mut(session_id) {
                // Record the full turn: question, answer, and commands
                session.add_turn(
                    question,
                    &ask_result.answer,
                    ask_result.commands_executed.clone(),
                );
            }
            // v0.0.905: Cache successful answers (only if not a clarification)
            if ask_result.success && !ask_result.needs_clarification && !ask_result.answer.is_empty()
            {
                state_guard.cache_answer(question_to_use, &ask_result.answer);
                debug!("Cached answer for: {}", question_to_use);
            }
            // Cleanup old sessions periodically (also triggers periodic save to disk)
            state_guard.cleanup_sessions();
        }
        Err(e) => {
            let response = StreamingResponse::Error {
                message: format!("Execution error: {}", e),
            };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
        }
    }

    Ok(())
}
