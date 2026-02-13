//! Finish streaming loop with final answer, verification, and recipe learning.

use anna_shared::config::AnnaConfig;
use anna_shared::exposure::ExposureGate;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::debug;

use crate::department;
use super::streaming_helpers::{build_final_answer_with_confidence, push_and_send, send_done};
use super::recipe_learning::{build_teaching_context, learn_recipe_from_answer};
use super::suggestions;
use super::verification::verify_answer;
use super::criteria::IterationState;

/// Finish successful streaming loop with final answer.
#[allow(clippy::too_many_arguments)]
pub async fn finish_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
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
    let config = AnnaConfig::load().ok();
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
        let explanation = anna_shared::teaching::generate_teaching(&teaching_ctx);
        anna_shared::teaching::format_teaching_block(&explanation, true)
    } else {
        None
    };

    // Build and send final answer (v0.3.113: with confidence indicator)
    let final_answer = build_final_answer_with_confidence(
        &verification.answer,
        &verification.evidence_line,
        teaching_block,
        Some(confidence),
    );
    push_and_send(writer, dialogue, StepType::FinalAnswer, final_answer.clone(), gate).await?;

    // Learn recipe and update ticket
    learn_recipe_from_answer(question, &state.commands, confidence);

    // v0.3.105: Also learn to Memory for semantic retrieval
    if confidence >= 0.7 && !state.commands.is_empty() {
        if let Ok(mut memory) = anna_shared::memory::Memory::load() {
            memory.learn(
                question,
                state.commands.clone(),
                &final_answer,
                anna_shared::memory::ExperienceContext::default(),
            );
            if let Err(e) = memory.save() {
                debug!("Failed to save memory: {}", e);
            } else {
                debug!("Learned experience to memory (confidence={:.2})", confidence);
            }
        }
    }

    let mut updated_ticket = ticket.clone();
    updated_ticket.resolve(&final_answer, 10);
    department::update_ticket(&updated_ticket);

    // v0.3.108: Generate proactive suggestions
    let suggestions = suggestions::generate_suggestions(question, &final_answer, &state.commands);
    let answer_with_suggestions = if let Some(suggestion_text) = suggestions::format_suggestions(&suggestions) {
        format!("{}{}", final_answer, suggestion_text)
    } else {
        final_answer.clone()
    };

    // Send done
    let result = AskResult {
        answer: answer_with_suggestions,
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
