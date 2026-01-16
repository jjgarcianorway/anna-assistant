//! Main question handling logic.
//! v0.1.1: Ralph loop integration
//! v0.2.8: RPG stats tracking
//! v0.3.56: Phase 23 - Outcome ledger integration
//! v0.3.59: Phase 26 - Abstention outcome recording
//! v0.3.70: Warning inquiry interception - DATA template routing
//! v0.3.71: Teaching Mode - intent classification and routing
//! v0.3.72: Interpretation Mode - resolution acknowledgment

use anna_shared::config::AnnaConfig;
use anna_shared::intent_class::classify_intent;
use anna_shared::interpretation::{
    is_resolution_inquiry, detect_resolutions, attribute_resolution,
    format_resolution_acknowledgment, format_no_resolution,
};
use anna_shared::monitor::{find_matching_issue, format_issue_evidence};
use anna_shared::outcome_ledger::{append_outcome, AbstentionReason, Outcome, OutcomeRecord, RequestMode};
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anna_shared::teaching::{classify_teaching_intent, TeachingIntent};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

use crate::core_loop::execute_question_streaming;
use crate::intent::is_warning_inquiry;
use crate::ralph;
use crate::state::SharedState;

use super::helpers::send_filtered_final_answer;

/// Handle main question processing (after all special cases)
pub async fn handle_main_question(
    question: &str,
    session_id: &str,
    state: SharedState,
    start_time: std::time::Instant,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    // v0.3.56: Phase 23 - Generate request ID and classify intent for outcome tracking
    let request_id = uuid::Uuid::new_v4().to_string();
    let intent = classify_intent(question);

    // v0.3.70: Check for warning inquiry FIRST - route to DATA template, not LLM
    // This is critical for Observation Phase compliance
    if let Some(subject) = is_warning_inquiry(question) {
        info!("Warning inquiry detected for subject: {}", subject);

        if let Some(issue) = find_matching_issue(&subject) {
            info!("Found matching issue, returning evidence-only response");

            // Format as pure evidence - NO LLM, NO explanation
            let evidence = format_issue_evidence(&issue);

            // Send the evidence as the answer
            send_filtered_final_answer(writer, &evidence).await?;

            let result = anna_shared::rpc::AskResult {
                answer: evidence,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(1.0), // Evidence is authoritative
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // Record outcome
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent.clone(),
                Outcome::Resolved,
                false,
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);

            return Ok(());
        } else {
            // Warning inquiry but no matching issue - report that clearly
            info!("Warning inquiry but no matching issue found");
            let no_issue_response = format!(
                "No active issue found matching '{}'.\n\n\
                 Current active issues can be viewed with: annactl status\n\n\
                 [No data to report]",
                subject
            );

            send_filtered_final_answer(writer, &no_issue_response).await?;

            let result = anna_shared::rpc::AskResult {
                answer: no_issue_response,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(1.0),
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent.clone(),
                Outcome::Resolved,
                false,
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);

            return Ok(());
        }
    }

    // v0.3.72: Interpretation Mode - check for resolution inquiry
    // Only respond about resolutions when explicitly asked
    if is_resolution_inquiry(question) {
        info!("Resolution inquiry detected: {}", question);

        // Detect any recent resolutions
        let resolutions = detect_resolutions();

        if !resolutions.is_empty() {
            // Find the most relevant resolution (most recent)
            let resolution = &resolutions[0];
            let attribution = attribute_resolution(resolution);

            info!(
                "Found resolution: {:?} attributed to {:?}",
                resolution.resolution, attribution.actor
            );

            // Format acknowledgment (strictly constrained output)
            let acknowledgment = format_resolution_acknowledgment(resolution, &attribution);

            send_filtered_final_answer(writer, &acknowledgment).await?;

            let result = anna_shared::rpc::AskResult {
                answer: acknowledgment,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(1.0),
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent.clone(),
                Outcome::Resolved,
                false,
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);

            return Ok(());
        } else {
            // No resolutions found - report that
            info!("Resolution inquiry but no resolutions detected");
            let no_resolution = format_no_resolution("requested subject");

            send_filtered_final_answer(writer, &no_resolution).await?;

            let result = anna_shared::rpc::AskResult {
                answer: no_resolution,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(1.0),
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let outcome_record = OutcomeRecord::new(
                &request_id,
                RequestMode::Dialogue,
                intent.clone(),
                Outcome::Resolved,
                false,
                duration_ms,
            );
            let _ = append_outcome(&outcome_record);

            return Ok(());
        }
    }

    // v0.3.71: Teaching Mode - classify intent for routing and logging
    let teaching_intent = classify_teaching_intent(question);
    let teaching_mode_enabled = AnnaConfig::load()
        .map(|c| c.teaching_mode)
        .unwrap_or(false);

    info!(
        "Teaching Mode: classified as {:?} (allows_teaching: {}, requires_evidence: {}, enabled: {})",
        teaching_intent,
        teaching_intent.allows_teaching(),
        teaching_intent.requires_evidence(),
        teaching_mode_enabled
    );

    // Teaching Mode behavior (only when enabled):
    // - Status/ChangeAnalysis: fact-based, grounded responses (always)
    // - Explanation/ServiceDesk: teaching allowed IF config.teaching_mode == true
    // - ActionRequest: routes to existing mutation flow (always)
    //
    // When Teaching Mode is disabled (default), all intents get Observation Phase behavior:
    // evidence only, no interpretation.
    //
    // Hard constraints remain regardless of mode:
    // - No new execution capabilities
    // - No unsolicited actions
    // - No guessing or hallucination
    // - No shell commands unless explicitly allowed

    // Store teaching context for use in Ralph loop (future: pass to LLM prompts)
    let _teaching_context = (teaching_intent, teaching_mode_enabled);

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

    // v0.1.1: Check if Ralph loop is enabled (simpler, more robust)
    let use_ralph = AnnaConfig::load()
        .map(|c| c.use_ralph_loop)
        .unwrap_or(true);

    let result = if use_ralph {
        info!("Using Ralph loop for question: {}", question_to_use);
        ralph::ralph_loop_streaming(&model, question_to_use, writer).await
    } else {
        execute_question_streaming(
            &model,
            question_to_use,
            session_context.as_deref(),
            writer,
        )
        .await
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
