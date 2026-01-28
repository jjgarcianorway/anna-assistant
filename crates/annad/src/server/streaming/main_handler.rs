//! Main question handling logic.
//! LLM-first: all questions go to the Ralph loop.

use anna_shared::config::AnnaConfig;
use anna_shared::intent_class::classify_intent;
use anna_shared::outcome_ledger::{append_outcome, AbstentionReason, Outcome, OutcomeRecord, RequestMode};
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

use crate::ralph;
use crate::state::SharedState;

use super::helpers::send_filtered_final_answer;

/// Handle main question processing - LLM-first, no bypass paths.
pub async fn handle_main_question(
    question: &str,
    session_id: &str,
    state: SharedState,
    start_time: std::time::Instant,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let intent = classify_intent(question);

    // Check cache for identical recent question
    {
        let state_guard = state.read().await;
        if let Some(cached_answer) = state_guard.get_cached_answer(question) {
            info!("Returning cached answer for: {}", question);
            // Phase 15: Filter cached answer through ExposureGate
            send_filtered_final_answer(writer, &cached_answer).await?;

            // Send done with AskResult
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
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // v0.3.56: Record outcome for cached answer
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent,
                Outcome::Resolved,
                false, // not escalated
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);

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

                // v0.3.56: Record failed outcome (daemon not ready)
                let duration_ms = start_time.elapsed().as_millis() as u64;
                let outcome_record = OutcomeRecord::new(
                    &request_id,
                    RequestMode::Dialogue,
                    intent,
                    Outcome::Failed,
                    false,
                    duration_ms,
                );
                let _ = append_outcome(&outcome_record);

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

            // Phase 15: Filter cached answer through ExposureGate
            send_filtered_final_answer(writer, &cached_answer).await?;

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
            let json = serde_json::to_string(&StreamingResponse::Done { result })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // v0.3.56: Record outcome for cached answer (expanded)
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent,
                Outcome::Resolved,
                false,
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);

            return Ok(());
        }
    }

    // LLM-first: all questions go through the Ralph loop
    info!("Ralph loop for question: {}", question_to_use);
    let result = ralph::ralph_loop_streaming(&model, question_to_use, writer).await;

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

            // v0.2.8: Record RPG stats
            let elapsed = start_time.elapsed();
            let response_ms = elapsed.as_millis() as u64;
            let answer_type = if ask_result.iterations == 0 {
                anna_shared::stats::AnswerType::Instant
            } else if ask_result.cached {
                anna_shared::stats::AnswerType::Memory
            } else {
                anna_shared::stats::AnswerType::Llm
            };
            if let Ok(mut stats) = anna_shared::stats::PersistentStats::load() {
                stats.record_answer(response_ms, answer_type);
                let _ = stats.save();
            }

            // v0.3.56: Phase 23 - Record outcome
            // v0.3.59: Phase 26 - Handle abstention
            // v0.3.60: Phase 27 - Record probes used
            let probes_used = if ask_result.commands_executed.is_empty() {
                None
            } else {
                Some(ask_result.commands_executed.clone())
            };

            if ask_result.abstained {
                let reason = AbstentionReason::LowConfidence {
                    final_confidence: ask_result.final_confidence.unwrap_or(0.0),
                    threshold: 0.5,
                };
                let mut outcome_record = OutcomeRecord::new_abstention(
                    &request_id,
                    RequestMode::Dialogue,
                    intent,
                    reason,
                    false,
                    response_ms,
                );
                outcome_record.probes_used = probes_used;
                let _ = append_outcome(&outcome_record);
            } else {
                let outcome = if ask_result.success {
                    Outcome::Resolved
                } else {
                    Outcome::Failed
                };
                let mut outcome_record = OutcomeRecord::new(
                    &request_id,
                    RequestMode::Dialogue,
                    intent,
                    outcome,
                    false, // TODO: track escalation from ask_result if available
                    response_ms,
                );
                outcome_record.probes_used = probes_used;
                let _ = append_outcome(&outcome_record);
            }
        }
        Err(e) => {
            // v0.3.30: Always attempt to send Error, don't propagate failures
            // This ensures the client gets a terminal response even if writes are flaky
            let response = StreamingResponse::Error {
                message: format!("Execution error: {}", e),
            };
            if let Ok(json) = serde_json::to_string(&response) {
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                let _ = writer.flush().await;
            }
            warn!("Streaming request failed: {}", e);

            // v0.3.56: Phase 23 - Record failed outcome
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent,
                Outcome::Failed,
                false,
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);
        }
    }

    // v0.3.30: Explicit flush before connection close
    let _ = writer.flush().await;
    Ok(())
}
