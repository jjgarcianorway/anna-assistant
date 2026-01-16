//! Streaming Ralph loop with real-time progress updates.
//! v0.3.46: All dialogue filtered through ExposureGate before emission.
//! v0.3.57: Phase 24 - Policy-driven behavior modulation.
//! v0.3.59: Phase 26 - Abstention tracking for low-confidence exits.
//! v0.3.69: Phase 28 - Capability question grounding (bypass LLM for capability Qs).

use anna_shared::declaration::CapabilityDeclaration;
use anna_shared::experiment::estimate_command_risk;
use anna_shared::exposure::ExposureGate;
use anna_shared::policy::get_policy;
use anna_shared::probe_ledger::ProbeLedger;
use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
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
use super::streaming_helpers::{build_final_answer, push_and_send, send_dialogue_steps, send_done, with_heartbeat};
use super::verification::{truncate, verify_answer};

/// Phase 28: Detect if question is asking about Anna's capabilities.
/// Ground capability answers in the declaration, not LLM.
fn is_capability_question(question: &str) -> bool {
    let q = question.to_lowercase();
    let patterns = [
        "what can you do",
        "what are your capabilities",
        "what commands can you run",
        "what are you capable of",
        "what can anna do",
        "what are anna's capabilities",
        "what is anna capable of",
        "tell me your capabilities",
        "list your capabilities",
        "show your capabilities",
        "what's your capability",
        "your capabilities",
        "what do you know how to do",
        "what are you able to do",
        "what are you allowed to do",
        "what is anna allowed to do",
        "allowed to do",
    ];
    patterns.iter().any(|p| q.contains(p))
}

/// Phase 28: Answer capability questions directly from the declaration.
async fn answer_capability_question<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
) -> Result<AskResult> {
    info!("Answering capability question from declaration (not LLM)");

    let decl = CapabilityDeclaration::from_ledger();
    let answer = decl.render_onboarding();
    let mut dialogue = Vec::new();

    let step = DialogueStep {
        step_type: StepType::FinalAnswer,
        content: answer.clone(),
    };
    dialogue.push(step.clone());
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    let result = AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(1.0),
    };

    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(result)
}

/// Streaming version of the Ralph loop with real-time progress updates.
pub async fn ralph_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
) -> Result<AskResult> {
    // v0.3.69: Phase 28 - Ground capability questions in declaration, not LLM
    if is_capability_question(question) {
        return answer_capability_question(writer).await;
    }

    // v0.3.46: Create ExposureGate for central filtering
    let gate = ExposureGate::from_config();

    // v0.3.57: Phase 24 - Log policy decisions in Debug mode
    if gate.diagnostic_visible() {
        let policy = get_policy();
        let basis = policy.format_debug_basis();
        debug!("{}", basis);
        // Emit policy basis as diagnostic step
        let step = DialogueStep {
            step_type: StepType::PolicyBasis,
            content: basis,
        };
        let _ = super::streaming_helpers::send_step(writer, step, &gate).await;
    }

    // Try instant error response first for known issues
    if let Some(result) = try_instant_error(question) {
        info!("Instant error response streaming completed");
        send_dialogue_steps(writer, &result.dialogue, &gate).await?;
        send_done(writer, &result).await?;
        return Ok(result);
    }

    // Try fast-path first for simple queries
    if let Some(mut result) = try_fast_path(question).await {
        info!("Fast-path streaming completed");
        send_dialogue_steps(writer, &result.dialogue, &gate).await?;
        push_and_send(writer, &mut result.dialogue, StepType::FinalAnswer, result.answer.clone(), &gate)
            .await?;
        send_done(writer, &result).await?;
        return Ok(result);
    }

    // Try diagnostic path for ambiguous queries (streaming)
    if let Some((executed, outputs, intro, mut dialogue)) = try_diagnostic_path(question) {
        info!("Diagnostic path streaming: analyzing {} outputs", outputs.len());
        send_dialogue_steps(writer, &dialogue, &gate).await?;

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
                push_and_send(writer, &mut dialogue, StepType::FinalAnswer, final_answer.clone(), &gate)
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
                    abstained: false,
                    final_confidence: None, // Diagnostic path doesn't track confidence
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
    run_full_loop_streaming(model, question, writer, &gate).await
}

/// Run the full Ralph loop with streaming progress.
async fn run_full_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<AskResult> {
    let criteria = determine_criteria(question);
    info!("Ralph streaming: {:?}, max {} iterations", criteria.answer_type, criteria.max_iterations);

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record and send user's question
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;

    // Create ticket for fly-on-the-wall experience
    let dept_name = department::determine_department(question);
    let mut ticket = department::create_ticket(question, dept_name);
    push_and_send(writer, &mut dialogue, StepType::TicketCreated, ticket.case_number.clone(), gate)
        .await?;

    // Dispatch to appropriate specialist
    let specialist = department::get_specialist_for_topic(question);
    let assigned_spec_name = if let Some(spec) = specialist {
        ticket.assign(spec.name);
        department::update_ticket(&ticket);
        let assignment = team_speak::anna_assigns_to(spec, question);
        push_and_send(writer, &mut dialogue, StepType::TeamAssignment, assignment, gate).await?;
        let ack = team_speak::specialist_acknowledges(spec);
        push_and_send(writer, &mut dialogue, StepType::SpecialistWorking,
            format!("{}: {}", spec.name, ack), gate).await?;
        Some(spec.name.to_string())
    } else {
        None
    };

    // Track investigation probes with deduplication (Phase 22)
    let mut probe_ledger = ProbeLedger::new();
    let mut probe_count: usize = 0;
    let mut experiment_count: usize = 0;

    // Start investigation mode
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart, question.to_string(), gate)
        .await?;

    ticket.start_investigating();
    department::update_ticket(&ticket);

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        debug!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        let commands = get_commands(model, question, &state).await?;

        for cmd in &commands {
            // Phase 22: Deduplicate probes via ProbeLedger
            if !probe_ledger.should_execute(cmd) {
                debug!("Skipping duplicate probe: {}", cmd);
                continue;
            }

            let risk = estimate_command_risk(cmd);
            let is_risky = risk > 0.3;

            probe_count += 1;
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe, cmd.clone(), gate)
                .await?;

            if is_risky {
                experiment_count += 1;
                ticket.start_experimenting();
                department::update_ticket(&ticket);
                push_and_send(writer, &mut dialogue, StepType::ExperimentStart,
                    format!("[risk={:.2}] expected=success", risk), gate).await?;
            }

            match execute_command(cmd) {
                Ok(output) => {
                    let clean_output = strip_ansi_codes(&output);
                    state.commands.push(cmd.clone());
                    state.outputs.push(clean_output.clone());
                    push_and_send(writer, &mut dialogue, StepType::InvestigationResult,
                        truncate(&clean_output, 500), gate).await?;
                    if is_risky {
                        let actual = if clean_output.contains("error") || clean_output.contains("failed")
                            { "failed" } else { "success" };
                        push_and_send(writer, &mut dialogue, StepType::ExperimentResult,
                            format!("actual={}", actual), gate).await?;
                        ticket.start_investigating();
                        department::update_ticket(&ticket);
                    }
                }
                Err(e) => {
                    if is_risky {
                        push_and_send(writer, &mut dialogue, StepType::ExperimentResult,
                            format!("actual=error ({})", e), gate).await?;
                        ticket.start_investigating();
                        department::update_ticket(&ticket);
                    }
                    state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                }
            }
        }

        // Phase 22: Wrap LLM calls with heartbeat emission
        let answer = with_heartbeat(writer, gate, generate_answer(model, question, &state, &criteria)).await?;
        state.answer = Some(answer.clone());

        let eval = with_heartbeat(writer, gate, self_evaluate(model, question, &answer, &state, &criteria)).await?;
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
                gate,
            )
            .await;
        }

        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
    }

    // Max iterations - return best effort
    push_and_send(writer, &mut dialogue, StepType::InvestigationComplete,
        format!("{} probes, {} experiments run (max iterations reached)", probe_count, experiment_count),
        gate).await?;

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
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, final_answer.clone(), gate).await?;

    // Phase 26: Determine if this is abstention vs failure
    // Abstention: max iterations + low confidence + no execution errors
    let has_execution_error = state.feedback.as_ref()
        .map(|f| f.contains("failed") || f.contains("error"))
        .unwrap_or(false);
    let is_abstained = state.confidence < 0.5 && !has_execution_error;

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
        abstained: is_abstained,
        final_confidence: Some(state.confidence),
    };
    send_done(writer, &result).await?;

    Ok(result)
}

/// Finish successful streaming loop with final answer.
#[allow(clippy::too_many_arguments)]
async fn finish_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W, dialogue: &mut Vec<DialogueStep>, ticket: &mut department::Ticket,
    state: &IterationState, answer: &str, question: &str, iteration: u32,
    probe_count: usize, experiment_count: usize, assigned_spec_name: &Option<String>,
    confidence: f32, gate: &ExposureGate,
) -> Result<AskResult> {
    // End investigation mode
    push_and_send(writer, dialogue, StepType::InvestigationComplete,
        format!("{} probes, {} experiments run", probe_count, experiment_count), gate).await?;

    // Specialist reports completion
    if let Some(ref spec_name) = assigned_spec_name {
        push_and_send(writer, dialogue, StepType::TeamDialogue,
            format!("{} -> Anna: I've got the answer.", spec_name), gate).await?;
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
    push_and_send(writer, dialogue, StepType::FinalAnswer, final_answer.clone(), gate).await?;

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
        abstained: false,
        final_confidence: Some(confidence),
    };
    send_done(writer, &result).await?;

    Ok(result)
}
