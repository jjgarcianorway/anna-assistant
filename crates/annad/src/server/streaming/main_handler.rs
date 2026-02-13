//! Main question handling logic.
//! LLM-first: all questions go to the Ralph loop.
//! v0.3.104: Multi-agent analysis for complexity-based routing.

use anna_shared::config::AnnaConfig;
use anna_shared::intent_class::classify_intent;
use anna_shared::outcome_ledger::{append_outcome, AbstentionReason, Outcome, OutcomeRecord, RequestMode};
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

use crate::orchestrator::TaskAnalysis;
use crate::ralph;
use crate::state::SharedState;

use super::helpers::send_filtered_final_answer;

/// Handle main question processing - LLM-first, no bypass paths.
pub async fn handle_main_question(
    original_question: &str,
    session_id: &str,
    state: SharedState,
    start_time: std::time::Instant,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let intent = classify_intent(original_question);

    // v0.3.179: Instant answers via dedicated module (bypasses LLM entirely)
    if super::instant_answers::try_instant_answer(original_question, writer, &state).await? {
        return Ok(());
    }

    // v0.3.180: Context resolution - detect missing references and resolve config files
    let username = crate::user_context::get_real_user().unwrap_or_else(|_| "root".to_string());
    let question = match crate::context_resolver::resolve_context(original_question, &username)? {
        crate::context_resolver::ContextResolution::NeedsClarification(clarification) => {
            info!("Missing context detected, sending clarification request");
            send_filtered_final_answer(writer, &clarification).await?;

            let result = anna_shared::rpc::AskResult {
                answer: clarification,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: true,
                clarification_question: Some("Please provide more specific information.".to_string()),
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
        crate::context_resolver::ContextResolution::Resolved { resolved_question, found_path, .. } => {
            info!("Config file auto-resolved: {} → {}", original_question, found_path.display());
            // Continue with resolved question that includes file path
            resolved_question
        }
        crate::context_resolver::ContextResolution::Clear => {
            // No issues, proceed with original question
            original_question.to_string()
        }
    };

    // From here on, use the resolved question
    let question = question.as_str();

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

    // Get session context (LLM will understand references naturally)
    // v0.3.138: Removed hardcoded expand_question() - let LLM handle it
    let session_context = {
        let mut state_guard = state.write().await;
        let session = state_guard.get_or_create_session(session_id);
        if session.history.is_empty() {
            None
        } else {
            Some(session.get_context_for_llm())
        }
    };

    // Get model from state
    let model = {
        let state_guard = state.read().await;
        match &state_guard.model {
            Some(m) => m.clone(),
            None => {
                // Stream a friendly first-run message instead of an error
                let init_status = state_guard.init_status.clone();
                drop(state_guard);

                let msg = format!(
                    "I'm still getting set up on your system — {}  \
                    Come back in a few minutes and I'll be ready to help.",
                    init_status
                );

                // Stream word by word so the client renders naturally
                for word in msg.split_whitespace() {
                    let token = StreamingResponse::Token { token: format!("{} ", word) };
                    let json = serde_json::to_string(&token)?;
                    writer.write_all(format!("{}\n", json).as_bytes()).await?;
                }

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
                writer.flush().await?;

                return Ok(());
            }
        }
    };

    // v0.0.905: Check answer cache before running LLM
    {
        let state_guard = state.read().await;
        if let Some(cached_answer) = state_guard.get_cached_answer(question) {
            info!("Returning cached answer for: {}", question);

            // Send cached response with dialogue showing it's cached
            let step = DialogueStep {
                step_type: StepType::UserQuestion,
                content: question.to_string(),
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

    // v0.3.108: Analyze task and optionally switch models based on complexity
    let config_result = AnnaConfig::load();
    let effective_model = if let Ok(ref config) = config_result {
        let analysis = TaskAnalysis::analyze(question, config);
        info!(
            "Task analysis: complexity={}, domains={:?}, multi_domain={}, recommended={}",
            analysis.complexity, analysis.domains, analysis.is_multi_domain, analysis.recommended_model
        );

        // Use recommended model if multi-agent mode is enabled
        if config.agents.multi_agent_mode && analysis.recommended_model != model {
            // Check if recommended model is available
            match crate::ollama::list_models().await {
                Ok(available) => {
                    let recommended = &analysis.recommended_model;
                    if available.iter().any(|m| m.starts_with(recommended.split(':').next().unwrap_or(""))) {
                        info!("Switching to {} model for {} task", recommended, analysis.complexity);
                        analysis.recommended_model.clone()
                    } else {
                        debug!("Recommended model {} not available, using default", recommended);
                        model.clone()
                    }
                }
                Err(e) => {
                    debug!("Could not list models: {}, using default", e);
                    model.clone()
                }
            }
        } else {
            model.clone()
        }
    } else {
        model.clone()
    };

    // v0.3.109: Check for parallel investigation opportunity
    let result = if let Ok(ref config) = config_result {
        if let Some(domains) = ralph::should_parallelize(question, config) {
            info!("Running parallel investigation for {} domains", domains.len());

            // Send parallel investigation indicator
            let step = DialogueStep {
                step_type: StepType::InvestigationStart,
                content: format!("Parallel investigation: {:?}", domains.iter().map(|d| d.as_str()).collect::<Vec<_>>()),
            };
            let json = serde_json::to_string(&StreamingResponse::Step { step })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // Run parallel investigation
            let max_parallel = config.agents.max_parallel_agents;
            let domain_results = ralph::run_parallel_investigation(
                &effective_model, question, domains, max_parallel
            ).await?;

            // Synthesize results
            let combined = ralph::synthesize_parallel_results(question, domain_results);

            // Send combined answer
            let final_step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: combined.answer.clone(),
            };
            let json = serde_json::to_string(&StreamingResponse::Step { step: final_step })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // Send done
            let done = StreamingResponse::Done { result: combined.clone() };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            Ok(combined)
        } else {
            // Single domain - use normal Ralph loop
            info!("Ralph loop for question: {} (model: {})", question, effective_model);
            ralph::ralph_loop_streaming(&effective_model, question, session_id, writer).await
        }
    } else {
        // No config - use normal Ralph loop
        info!("Ralph loop for question: {} (model: {})", question, effective_model);
        ralph::ralph_loop_streaming(&effective_model, question, session_id, writer).await
    };

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
                state_guard.cache_answer(question, &ask_result.answer);
                debug!("Cached answer for: {}", question);
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
