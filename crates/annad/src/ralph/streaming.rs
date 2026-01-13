//! Streaming version of the Ralph loop.
//! Sends progress updates to the client in real-time.

use anna_shared::experiment::estimate_command_risk;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anna_shared::teaching;
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{execute_command, strip_ansi_codes};
use crate::department;
use crate::ollama;
use crate::team_speak;

use super::commands::{generate_answer, get_commands, self_evaluate};
use super::criteria::{determine_criteria, IterationState};
use super::diagnostic::try_diagnostic_path;
use super::fast_path::try_fast_path;
use super::instant::try_instant_error;
use super::recipe_learning::{build_teaching_context, learn_recipe_from_answer};
use super::streaming_helpers::{build_final_answer, push_and_send, send_dialogue_steps, send_done};
use super::verification::{truncate, verify_answer};

/// Streaming version of the Ralph loop with real-time progress updates.
pub async fn ralph_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
) -> Result<AskResult> {
    // Try instant error response first for known issues
    if let Some(result) = try_instant_error(question) {
        info!("Instant error response streaming completed");
        send_dialogue_steps(writer, &result.dialogue).await?;
        send_done(writer, &result).await?;
        return Ok(result);
    }

    // Try fast-path first for simple queries
    if let Some(mut result) = try_fast_path(question).await {
        info!("Fast-path streaming completed");
        send_dialogue_steps(writer, &result.dialogue).await?;
        push_and_send(writer, &mut result.dialogue, StepType::FinalAnswer, result.answer.clone())
            .await?;
        send_done(writer, &result).await?;
        return Ok(result);
    }

    // Try diagnostic path for ambiguous queries (streaming)
    if let Some((executed, outputs, intro, mut dialogue)) = try_diagnostic_path(question) {
        info!("Diagnostic path streaming: analyzing {} outputs", outputs.len());
        send_dialogue_steps(writer, &dialogue).await?;

        let data_context = outputs.join("\n---\n");
        let prompt = format!(
            r#"You are Anna, an AI assistant for Arch Linux systems.

The user asked: "{}"

I ran diagnostic commands. Here are the results:
{}

Based on these diagnostics, provide a helpful analysis. Be specific:
- If there's a problem, explain what it is and how to fix it
- If everything looks normal, say so with specific evidence
- Reference actual values from the output

Be concise but complete. Start your response with "{}" (without quotes)."#,
            question, data_context, intro
        );

        match ollama::chat_with_timeout(model, &prompt, 60).await {
            Ok(answer) => {
                let evidence: Vec<(String, String, i32)> = executed
                    .iter()
                    .zip(outputs.iter())
                    .map(|(cmd, out)| (cmd.clone(), out.clone(), 0))
                    .collect();
                let debug_mode = anna_shared::config::AnnaConfig::load()
                    .map(|c| c.debug_mode)
                    .unwrap_or(false);
                let verification = verify_answer(&answer, question, &evidence, debug_mode);

                let final_answer =
                    build_final_answer(&verification.answer, &verification.evidence_line, None);
                push_and_send(writer, &mut dialogue, StepType::FinalAnswer, final_answer.clone())
                    .await?;

                let result = AskResult {
                    answer: final_answer,
                    success: true,
                    iterations: 1,
                    commands_executed: executed,
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                };
                send_done(writer, &result).await?;
                return Ok(result);
            }
            Err(e) => {
                warn!("Diagnostic path LLM failed: {}, falling back to normal loop", e);
            }
        }
    }

    // Full Ralph loop with streaming
    run_full_loop_streaming(model, question, writer).await
}

/// Run the full Ralph loop with streaming progress.
async fn run_full_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
) -> Result<AskResult> {
    let criteria = determine_criteria(question);
    info!("Ralph streaming: {:?}, max {} iterations", criteria.answer_type, criteria.max_iterations);

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record and send user's question
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string()).await?;

    // Create ticket for fly-on-the-wall experience
    let dept_name = department::determine_department(question);
    let mut ticket = department::create_ticket(question, dept_name);
    push_and_send(writer, &mut dialogue, StepType::TicketCreated, ticket.case_number.clone())
        .await?;

    // Dispatch to appropriate specialist
    let specialist = department::get_specialist_for_topic(question);
    let assigned_spec_name = if let Some(spec) = specialist {
        ticket.assign(spec.name);
        department::update_ticket(&ticket);
        let assignment = team_speak::anna_assigns_to(spec, question);
        push_and_send(writer, &mut dialogue, StepType::TeamAssignment, assignment).await?;
        let ack = team_speak::specialist_acknowledges(spec);
        push_and_send(
            writer,
            &mut dialogue,
            StepType::SpecialistWorking,
            format!("{}: {}", spec.name, ack),
        )
        .await?;
        Some(spec.name.to_string())
    } else {
        None
    };

    // Track investigation probes
    let mut probe_count: usize = 0;
    let mut experiment_count: usize = 0;

    // Start investigation mode
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart, question.to_string())
        .await?;

    ticket.start_investigating();
    department::update_ticket(&ticket);

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        debug!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        let commands = get_commands(model, question, &state).await?;

        for cmd in &commands {
            let risk = estimate_command_risk(cmd);
            let is_risky = risk > 0.3;

            probe_count += 1;
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe, cmd.clone())
                .await?;

            if is_risky {
                experiment_count += 1;
                ticket.start_experimenting();
                department::update_ticket(&ticket);
                push_and_send(
                    writer,
                    &mut dialogue,
                    StepType::ExperimentStart,
                    format!("[risk={:.2}] expected=success", risk),
                )
                .await?;
            }

            match execute_command(cmd) {
                Ok(output) => {
                    let clean_output = strip_ansi_codes(&output);
                    state.commands.push(cmd.clone());
                    state.outputs.push(clean_output.clone());
                    push_and_send(
                        writer,
                        &mut dialogue,
                        StepType::InvestigationResult,
                        truncate(&clean_output, 500),
                    )
                    .await?;

                    if is_risky {
                        let actual =
                            if clean_output.contains("error") || clean_output.contains("failed") {
                                "failed"
                            } else {
                                "success"
                            };
                        push_and_send(
                            writer,
                            &mut dialogue,
                            StepType::ExperimentResult,
                            format!("actual={}", actual),
                        )
                        .await?;
                        ticket.start_investigating();
                        department::update_ticket(&ticket);
                    }
                }
                Err(e) => {
                    if is_risky {
                        push_and_send(
                            writer,
                            &mut dialogue,
                            StepType::ExperimentResult,
                            format!("actual=error ({})", e),
                        )
                        .await?;
                        ticket.start_investigating();
                        department::update_ticket(&ticket);
                    }
                    state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                }
            }
        }

        let answer = generate_answer(model, question, &state, &criteria).await?;
        state.answer = Some(answer.clone());

        let eval = self_evaluate(model, question, &answer, &state, &criteria).await?;
        state.confidence = eval.confidence;

        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            return finish_streaming(
                writer,
                &mut dialogue,
                &mut ticket,
                &state,
                &answer,
                question,
                iteration,
                probe_count,
                experiment_count,
                &assigned_spec_name,
                eval.confidence,
            )
            .await;
        }

        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
    }

    // Max iterations - return best effort
    push_and_send(
        writer,
        &mut dialogue,
        StepType::InvestigationComplete,
        format!("{} probes, {} experiments run (max iterations reached)", probe_count, experiment_count),
    )
    .await?;

    let raw_answer = state.answer.unwrap_or_else(|| {
        "I couldn't fully answer your question. Please try rephrasing.".to_string()
    });
    let evidence: Vec<(String, String, i32)> = state
        .commands
        .iter()
        .zip(state.outputs.iter())
        .map(|(cmd, out)| (cmd.clone(), out.clone(), 0))
        .collect();
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &evidence, debug_mode);
    let final_answer =
        build_final_answer(&verification.answer, &verification.evidence_line, None);
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, final_answer.clone()).await?;

    let result = AskResult {
        answer: final_answer,
        success: state.confidence >= 0.5,
        iterations: iteration,
        commands_executed: state.commands,
        dialogue,
        needs_clarification: state.confidence < 0.3,
        clarification_question: state.not_done_reason,
        cached: false,
        citations: vec![],
    };
    send_done(writer, &result).await?;

    Ok(result)
}

/// Finish successful streaming loop with final answer.
#[allow(clippy::too_many_arguments)]
async fn finish_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    dialogue: &mut Vec<DialogueStep>,
    ticket: &mut department::Ticket,
    state: &IterationState,
    answer: &str,
    question: &str,
    iteration: u32,
    probe_count: usize,
    experiment_count: usize,
    assigned_spec_name: &Option<String>,
    confidence: f32,
) -> Result<AskResult> {
    // End investigation mode
    push_and_send(
        writer,
        dialogue,
        StepType::InvestigationComplete,
        format!("{} probes, {} experiments run", probe_count, experiment_count),
    )
    .await?;

    // Specialist reports completion
    if let Some(ref spec_name) = assigned_spec_name {
        push_and_send(
            writer,
            dialogue,
            StepType::TeamDialogue,
            format!("{} -> Anna: I've got the answer.", spec_name),
        )
        .await?;
    }

    // Verify answer through ClaimGate
    let evidence: Vec<(String, String, i32)> = state
        .commands
        .iter()
        .zip(state.outputs.iter())
        .map(|(cmd, out)| (cmd.clone(), out.clone(), 0))
        .collect();
    let config = anna_shared::config::AnnaConfig::load().ok();
    let debug_mode = config.as_ref().map(|c| c.debug_mode).unwrap_or(false);
    let teaching_mode = config.as_ref().map(|c| c.teaching_mode).unwrap_or(false);
    let verification = verify_answer(answer, question, &evidence, debug_mode);

    // Build teaching context
    let teaching_ctx = build_teaching_context(
        question,
        &state.commands,
        &state.outputs,
        experiment_count > 0,
        &verification.doc_citations,
    );

    // Generate teaching explanation if enabled
    let teaching_block = if teaching_mode {
        let explanation = teaching::generate_teaching(&teaching_ctx);
        teaching::format_teaching_block(&explanation, true)
    } else {
        None
    };

    // Build and send final answer
    let final_answer =
        build_final_answer(&verification.answer, &verification.evidence_line, teaching_block);
    push_and_send(writer, dialogue, StepType::FinalAnswer, final_answer.clone()).await?;

    // Learn recipe and update ticket
    learn_recipe_from_answer(question, &state.commands, confidence);
    let mut updated_ticket = ticket.clone();
    updated_ticket.resolve(&final_answer, 10);
    department::update_ticket(&updated_ticket);

    // Send done
    let result = AskResult {
        answer: final_answer,
        success: true,
        iterations: iteration,
        commands_executed: state.commands.clone(),
        dialogue: dialogue.clone(),
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    };
    send_done(writer, &result).await?;

    Ok(result)
}
