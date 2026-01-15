//! Streaming execution for LLM core

use anna_shared::exposure::{DialogueClassification, ExposureGate};
use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::info;

use super::evidence::verify_answer;
use super::types::{Finding, InvestigationState, NextStep};
use super::{decide_next_step, generate_answer, understand_question, MAX_ITERATIONS};
use crate::core_loop::command::execute_command;

/// Streaming version of execute_question
/// session_context is accepted for API compatibility but not currently used
pub async fn execute_question_streaming_llm<W: AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    _session_context: Option<&str>,
    writer: &mut W,
) -> Result<AskResult> {
    info!("LLM Core (streaming): Processing question: {}", question);

    let mut state = InvestigationState::default();
    let mut dialogue = Vec::new();
    let start_time = std::time::Instant::now();

    // v0.3.46: Use ExposureGate for central filtering
    let exposure_gate = ExposureGate::from_config();

    // PHASE 1: UNDERSTAND
    send_dialogue_gated(
        writer, "Anna", None,
        &format!("New request: \"{}\"", question),
        start_time, &exposure_gate, DialogueClassification::Informational
    ).await?;

    send_step(writer, DialogueStep {
        step_type: StepType::AnnaToLlm,
        content: "Understanding question...".to_string(),
    }, &mut dialogue).await?;

    let understanding = understand_question(model, question).await?;

    if let Some(reason) = &understanding.out_of_scope_reason {
        return finish_out_of_scope(writer, reason, &dialogue).await;
    }

    // PHASE 2: INVESTIGATE
    send_dialogue_gated(
        writer, "Anna", Some("Analyst"),
        "Running diagnostics...",
        start_time, &exposure_gate, DialogueClassification::Procedural
    ).await?;

    loop {
        state.iteration += 1;
        if state.iteration > MAX_ITERATIONS {
            send_dialogue_gated(
                writer, "Analyst", Some("Anna"),
                "Max iterations reached, compiling results.",
                start_time, &exposure_gate, DialogueClassification::Informational
            ).await?;
            break;
        }

        send_step(writer, DialogueStep {
            step_type: StepType::AnnaToLlm,
            content: format!("Investigation iteration {}...", state.iteration),
        }, &mut dialogue).await?;

        let next = decide_next_step(model, question, &state).await?;

        match next {
            NextStep::Investigate(commands) => {
                investigate_commands(
                    writer, &mut state, &mut dialogue,
                    commands, start_time, &exposure_gate
                ).await?;
            }
            NextStep::Answer => {
                send_dialogue_gated(
                    writer, "Analyst", Some("Anna"),
                    "Enough data gathered. Ready to answer.",
                    start_time, &exposure_gate, DialogueClassification::Informational
                ).await?;
                break;
            }
            NextStep::SuggestFix { problem, fix_command, explanation } => {
                return finish_with_fix(
                    writer, &state, &mut dialogue, question,
                    &problem, &fix_command, &explanation
                ).await;
            }
            NextStep::OutOfScope(reason) => {
                return finish_out_of_scope_with_state(
                    writer, &reason, &state, &dialogue
                ).await;
            }
        }
    }

    // PHASE 3: GENERATE ANSWER
    finish_with_answer(writer, model, question, &state, &mut dialogue, start_time, &exposure_gate).await
}

/// Helper to send streaming updates
async fn send_step<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
    dialogue: &mut Vec<DialogueStep>,
) -> Result<()> {
    dialogue.push(step.clone());
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// v0.3.46: Helper to send internal comms dialogue through ExposureGate
async fn send_dialogue_gated<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    speaker: &str,
    recipient: Option<&str>,
    message: &str,
    start_time: std::time::Instant,
    gate: &ExposureGate,
    classification: DialogueClassification,
) -> Result<()> {
    // Filter through ExposureGate - central enforcement
    let result = gate.filter(message, classification);
    if !result.emit {
        return Ok(());
    }
    let offset_ms = start_time.elapsed().as_millis() as u64;
    let response = StreamingResponse::Dialogue {
        speaker: speaker.to_string(),
        recipient: recipient.map(|s| s.to_string()),
        message: result.content,
        offset_ms,
    };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// Execute investigation commands
async fn investigate_commands<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    state: &mut InvestigationState,
    dialogue: &mut Vec<DialogueStep>,
    commands: Vec<String>,
    start_time: std::time::Instant,
    exposure_gate: &ExposureGate,
) -> Result<()> {
    send_dialogue_gated(
        writer, "Analyst", Some("Anna"),
        &format!("Running {} probe(s).", commands.len()),
        start_time, exposure_gate, DialogueClassification::Procedural
    ).await?;

    for cmd in commands {
        // Show command being executed
        send_dialogue_gated(
            writer, "Analyst", None,
            &format!("[probe] {}", cmd),
            start_time, exposure_gate, DialogueClassification::Procedural
        ).await?;

        send_step(writer, DialogueStep {
            step_type: StepType::CommandExec,
            content: cmd.clone(),
        }, dialogue).await?;

        // Execute command
        let cmd_clone = cmd.clone();
        let result = tokio::task::spawn_blocking(move || {
            execute_command(&cmd_clone)
        }).await?;

        let (output, success) = match result {
            Ok(out) => (out, true),
            Err(e) => (format!("Error: {}", e), false),
        };

        // Show truncated output
        let display_output = if output.len() > 500 {
            format!("{}...(truncated)", &output[..500])
        } else {
            output.clone()
        };

        send_step(writer, DialogueStep {
            step_type: StepType::CommandOutput,
            content: display_output,
        }, dialogue).await?;

        state.findings.push(Finding { command: cmd, output, success });
    }

    Ok(())
}

/// Finish with out of scope response
async fn finish_out_of_scope<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    reason: &str,
    dialogue: &[DialogueStep],
) -> Result<AskResult> {
    let result = AskResult {
        answer: format!("I'm Anna, your Linux assistant. {}", reason),
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: dialogue.to_vec(),
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    };
    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(result)
}

/// Finish with out of scope response (with state)
async fn finish_out_of_scope_with_state<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    reason: &str,
    state: &InvestigationState,
    dialogue: &[DialogueStep],
) -> Result<AskResult> {
    let result = AskResult {
        answer: format!("I'm Anna, your Linux assistant. {}", reason),
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue: dialogue.to_vec(),
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    };
    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(result)
}

/// Finish with suggested fix
async fn finish_with_fix<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    state: &InvestigationState,
    dialogue: &mut Vec<DialogueStep>,
    question: &str,
    problem: &str,
    fix_command: &str,
    explanation: &str,
) -> Result<AskResult> {
    let raw_answer = format!(
        "I found the issue: {}\n\n\
         I can fix this by running:\n  {}\n\n\
         {}\n\n\
         Would you like me to do this? (yes/no)",
        problem, fix_command, explanation
    );

    // v0.3.25: Verify answer through ClaimGate with evidence line
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

    // Append evidence line
    let final_answer = if verification.evidence_line.is_empty() {
        verification.answer.clone()
    } else {
        format!("{}\n\n{}", verification.answer, verification.evidence_line)
    };

    // v0.3.22: Truth-first rendering - send complete answer, no fake streaming
    send_step(writer, DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    }, dialogue).await?;

    let result = AskResult {
        answer: final_answer,
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue: dialogue.clone(),
        needs_clarification: true,
        clarification_question: Some("Confirm fix?".to_string()),
        cached: false,
        citations: vec![],
    };
    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(result)
}

/// Finish with generated answer
async fn finish_with_answer<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    model: &str,
    question: &str,
    state: &InvestigationState,
    dialogue: &mut Vec<DialogueStep>,
    start_time: std::time::Instant,
    exposure_gate: &ExposureGate,
) -> Result<AskResult> {
    send_dialogue_gated(
        writer, "Anna", None,
        "Generating final answer...",
        start_time, exposure_gate, DialogueClassification::Informational
    ).await?;

    send_step(writer, DialogueStep {
        step_type: StepType::AnnaToLlm,
        content: "Generating answer...".to_string(),
    }, dialogue).await?;

    let raw_answer = generate_answer(model, question, state).await?;

    send_dialogue_gated(
        writer, "Anna", None,
        "Answer ready.",
        start_time, exposure_gate, DialogueClassification::Informational
    ).await?;

    // v0.3.25: Verify answer through ClaimGate with evidence line
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

    // Append evidence line if we have any findings
    let final_answer = if verification.evidence_line.is_empty() {
        verification.answer.clone()
    } else {
        format!("{}\n\n{}", verification.answer, verification.evidence_line)
    };

    // v0.3.22: Truth-first rendering - send complete answer, no fake streaming
    send_step(writer, DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    }, dialogue).await?;

    let result = AskResult {
        answer: final_answer,
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue: dialogue.clone(),
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    };

    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(result)
}
