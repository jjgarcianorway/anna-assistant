//! Ralph-style autonomous iteration loop for answering questions.
//!
//! The Ralph Wiggum approach: iteration beats perfection.
//! Instead of complex branching, use a simple loop with clear completion criteria.
//!
//! Principles:
//! 1. Define "done" upfront - what does success look like?
//! 2. Iterate until done - trust the loop, not complexity
//! 3. Self-evaluate - LLM checks its own work before declaring done
//! 4. Learn from attempts - each iteration improves the next

mod commands;
pub mod confidence;
mod criteria;
mod diagnostic;
pub mod evidence;
mod fast_path;
mod fast_path_patterns;
mod instant;
mod recipe_learning;
mod streaming;
pub mod streaming_helpers;
mod verification;

// Re-export public API
pub use criteria::{determine_criteria, AnswerType, CompletionCriteria};
pub use streaming::ralph_loop_streaming;

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{execute_command, strip_ansi_codes};
use crate::ollama;

use commands::{generate_answer, get_commands, self_evaluate};
use criteria::IterationState;
use diagnostic::try_diagnostic_path;
use fast_path::try_fast_path;
use instant::try_instant_error;
use recipe_learning::learn_recipe_from_answer;
use verification::truncate;

/// The Ralph loop: iterate until done (non-streaming version)
///
/// This is the core of the Ralph approach:
/// 1. Determine what "done" looks like
/// 2. Loop: attempt answer, self-evaluate, improve
/// 3. Stop when criteria met or max iterations reached
pub async fn ralph_loop(model: &str, question: &str) -> Result<AskResult> {
    // Try instant error response first for known issues
    if let Some(result) = try_instant_error(question) {
        info!("Instant error response completed");
        return Ok(result);
    }

    // Try fast-path first for simple queries
    if let Some(result) = try_fast_path(question).await {
        info!("Fast-path completed in 0 iterations");
        return Ok(result);
    }

    // Try diagnostic path for ambiguous queries
    if let Some((executed, outputs, intro, mut dialogue)) = try_diagnostic_path(question) {
        info!("Diagnostic path: analyzing {} outputs", outputs.len());

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
                dialogue.push(DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: answer.clone(),
                });

                return Ok(AskResult {
                    answer,
                    success: true,
                    iterations: 1,
                    commands_executed: executed,
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                    abstained: false,
                    final_confidence: None,
                });
            }
            Err(e) => {
                warn!("Diagnostic path LLM failed: {}, falling back to normal loop", e);
            }
        }
    }

    let criteria = determine_criteria(question);
    info!(
        "Ralph loop: {:?}, confidence >= {:.0}%, max {} iterations",
        criteria.answer_type,
        criteria.min_confidence * 100.0,
        criteria.max_iterations
    );

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record the question
    dialogue.push(DialogueStep {
        step_type: StepType::UserQuestion,
        content: question.to_string(),
    });

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        info!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        // Step 1: Get commands to run
        let commands = get_commands(model, question, &state).await?;

        if commands.is_empty() && state.outputs.is_empty() {
            debug!("No commands needed, generating direct answer");
        } else if !commands.is_empty() {
            for cmd in &commands {
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandExec,
                    content: cmd.clone(),
                });

                match execute_command(cmd) {
                    Ok(output) => {
                        let clean_output = strip_ansi_codes(&output);
                        state.commands.push(cmd.clone());
                        state.outputs.push(clean_output.clone());
                        dialogue.push(DialogueStep {
                            step_type: StepType::CommandOutput,
                            content: truncate(&clean_output, 500),
                        });
                    }
                    Err(e) => {
                        debug!("Command failed: {}: {}", cmd, e);
                        state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                    }
                }
            }
        }

        // Step 2: Generate answer
        let answer = generate_answer(model, question, &state, &criteria).await?;
        state.answer = Some(answer.clone());

        // Step 3: Self-evaluate
        let eval = self_evaluate(model, question, &answer, &state, &criteria).await?;
        state.confidence = eval.confidence;

        debug!(
            "Self-evaluation: complete={}, confidence={:.0}%",
            eval.is_complete,
            eval.confidence * 100.0
        );

        // Step 4: Check completion criteria
        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            info!(
                "Ralph done! Confidence {:.0}% >= {:.0}% threshold",
                eval.confidence * 100.0,
                criteria.min_confidence * 100.0
            );

            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.clone(),
            });

            // Learn recipe from successful answer
            learn_recipe_from_answer(question, &state.commands, eval.confidence);

            return Ok(AskResult {
                answer,
                success: true,
                iterations: iteration,
                commands_executed: state.commands,
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(eval.confidence),
            });
        }

        // Not done yet - prepare feedback for next iteration
        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
        info!(
            "Not done yet: {:?}",
            state.not_done_reason.as_deref().unwrap_or("confidence too low")
        );
    }

    // Max iterations reached - return best effort
    warn!(
        "Ralph max iterations reached, returning best effort (confidence: {:.0}%)",
        state.confidence * 100.0
    );

    let final_answer = state.answer.unwrap_or_else(|| {
        "I wasn't able to fully answer your question. Please try rephrasing or ask about something more specific.".to_string()
    });

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    });

    // Phase 26: Determine if this is abstention vs failure
    let has_execution_error = state.feedback.as_ref()
        .map(|f| f.contains("failed") || f.contains("error"))
        .unwrap_or(false);
    let is_abstained = state.confidence < 0.5 && !has_execution_error;

    Ok(AskResult {
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
    })
}
