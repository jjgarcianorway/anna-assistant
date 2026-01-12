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
//!
//! v0.1.1: Initial implementation

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{
    execute_command, strip_ansi_codes,
};
use crate::ollama;

/// Completion criteria for a question
#[derive(Debug, Clone)]
pub struct CompletionCriteria {
    /// What type of answer is expected
    pub answer_type: AnswerType,
    /// Minimum confidence threshold (0.0 - 1.0)
    pub min_confidence: f32,
    /// Maximum iterations before giving up
    pub max_iterations: u32,
    /// Whether grounding in command output is required
    pub requires_grounding: bool,
}

impl Default for CompletionCriteria {
    fn default() -> Self {
        Self {
            answer_type: AnswerType::Factual,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
        }
    }
}

/// Types of answers Anna can provide
#[derive(Debug, Clone)]
pub enum AnswerType {
    /// Factual information from the system (requires command output)
    Factual,
    /// How-to instructions (may cite wiki/docs)
    HowTo,
    /// Troubleshooting help (requires diagnosis)
    Troubleshoot,
    /// Simple acknowledgment or clarification
    Simple,
}

/// State of an iteration attempt
#[derive(Debug)]
struct IterationState {
    /// Commands executed so far
    commands: Vec<String>,
    /// Outputs collected
    outputs: Vec<String>,
    /// Current answer draft
    answer: Option<String>,
    /// Confidence in current answer
    confidence: f32,
    /// Feedback from previous iteration
    feedback: Option<String>,
    /// Why we're not done yet
    not_done_reason: Option<String>,
}

impl Default for IterationState {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            outputs: Vec::new(),
            answer: None,
            confidence: 0.0,
            feedback: None,
            not_done_reason: None,
        }
    }
}

/// Quick quality check for answers (no LLM needed)
fn quick_quality_check(answer: &str) -> bool {
    let answer = answer.trim();

    // Too short
    if answer.len() < 10 {
        return false;
    }

    // Obvious refusals
    let refusals = ["i cannot", "i can't", "i'm not able", "i don't know"];
    if refusals.iter().any(|r| answer.to_lowercase().contains(r)) {
        return false;
    }

    // Prompt leakage
    let leakage = ["as an ai", "as a language model", "i'm an ai"];
    if leakage.iter().any(|l| answer.to_lowercase().contains(l)) {
        return false;
    }

    true
}

/// Result of self-evaluation
#[derive(Debug)]
struct SelfEvaluation {
    /// Is the answer complete?
    is_complete: bool,
    /// Confidence score (0.0 - 1.0)
    confidence: f32,
    /// What's missing if not complete
    missing: Option<String>,
    /// Suggestions for improvement
    suggestions: Option<String>,
}

/// Determine completion criteria based on the question
pub fn determine_criteria(question: &str) -> CompletionCriteria {
    let q = question.to_lowercase();

    // HowTo questions - instructions, don't need live output
    if q.contains("how do i")
        || q.contains("how to")
        || q.contains("how can i")
        || q.starts_with("install")
        || q.starts_with("setup")
        || q.starts_with("configure")
    {
        return CompletionCriteria {
            answer_type: AnswerType::HowTo,
            min_confidence: 0.6,
            max_iterations: 3,
            requires_grounding: false, // Instructions don't need live data
        };
    }

    // Troubleshooting - needs diagnosis
    if q.contains("not working")
        || q.contains("error")
        || q.contains("failed")
        || q.contains("problem")
        || q.contains("broken")
        || q.contains("fix")
        || q.contains("why")
    {
        return CompletionCriteria {
            answer_type: AnswerType::Troubleshoot,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
        };
    }

    // Simple questions
    if q.len() < 30 && !q.contains("?") {
        return CompletionCriteria {
            answer_type: AnswerType::Simple,
            min_confidence: 0.5,
            max_iterations: 2,
            requires_grounding: false,
        };
    }

    // Default: Factual query
    CompletionCriteria {
        answer_type: AnswerType::Factual,
        min_confidence: 0.7,
        max_iterations: 5,
        requires_grounding: true,
    }
}

/// The Ralph loop: iterate until done
///
/// This is the core of the Ralph approach:
/// 1. Determine what "done" looks like
/// 2. Loop: attempt answer, self-evaluate, improve
/// 3. Stop when criteria met or max iterations reached
pub async fn ralph_loop(model: &str, question: &str) -> Result<AskResult> {
    let criteria = determine_criteria(question);
    info!(
        "Ralph loop: {:?}, confidence >= {:.0}%, max {} iterations",
        criteria.answer_type, criteria.min_confidence * 100.0, criteria.max_iterations
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

        // Step 1: Get commands to run (or more commands if we have feedback)
        let commands = get_commands(model, question, &state).await?;

        if commands.is_empty() && state.outputs.is_empty() {
            // No commands needed - generate direct answer
            debug!("No commands needed, generating direct answer");
        } else if !commands.is_empty() {
            // Execute commands
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
                            content: if clean_output.len() > 500 {
                                format!("{}...(truncated)", &clean_output[..500])
                            } else {
                                clean_output
                            },
                        });
                    }
                    Err(e) => {
                        debug!("Command failed: {}: {}", cmd, e);
                        state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                    }
                }
            }
        }

        // Step 2: Generate answer based on collected data
        let answer = generate_answer(model, question, &state, &criteria).await?;
        state.answer = Some(answer.clone());

        // Step 3: Self-evaluate - is this answer good enough?
        let eval = self_evaluate(model, question, &answer, &state, &criteria).await?;
        state.confidence = eval.confidence;

        debug!(
            "Self-evaluation: complete={}, confidence={:.0}%",
            eval.is_complete, eval.confidence * 100.0
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

            return Ok(AskResult {
                answer,
                success: true,
                iterations: iteration,
                commands_executed: state.commands,
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
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

    Ok(AskResult {
        answer: final_answer,
        success: state.confidence >= 0.5,
        iterations: iteration,
        commands_executed: state.commands,
        dialogue,
        needs_clarification: state.confidence < 0.3, // v0.1.6: Lowered from 0.5 to reduce note spam
        clarification_question: state.not_done_reason,
        cached: false,
    })
}

/// Get commands to run for answering the question
async fn get_commands(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<Vec<String>> {
    let feedback_context = if let Some(ref feedback) = state.feedback {
        format!(
            "\n\nPrevious attempt feedback: {}\nAlready tried: {:?}",
            feedback, state.commands
        )
    } else {
        String::new()
    };

    let output_context = if !state.outputs.is_empty() {
        format!(
            "\n\nData collected so far:\n{}",
            state.outputs.join("\n---\n")
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"Question: {}{}{}

What Linux commands should I run to answer this question?
- Only suggest commands that will help answer the specific question
- Use standard tools (cat, grep, systemctl, pacman, etc.)
- If no commands are needed, output NONE

Output format: one command per line, nothing else.
If sufficient data is collected, output DONE."#,
        question, output_context, feedback_context
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    let response = response.trim();

    // Check for special responses (case-insensitive)
    let response_upper = response.to_uppercase();
    if response_upper == "NONE" || response_upper == "DONE" || response.is_empty() {
        return Ok(Vec::new());
    }

    let commands: Vec<String> = response
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            if l.is_empty() || l.starts_with('#') {
                return false;
            }
            // Filter out DONE/NONE even if mixed with other commands
            let upper = l.to_uppercase();
            if upper == "DONE" || upper == "NONE" || upper.starts_with("DONE:") {
                return false;
            }
            true
        })
        .map(|l| l.to_string())
        .take(5) // Max 5 commands per iteration
        .collect();

    Ok(commands)
}

/// Generate an answer based on collected data
async fn generate_answer(
    model: &str,
    question: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
) -> Result<String> {
    let data_context = if state.outputs.is_empty() {
        "No command output available.".to_string()
    } else {
        state.outputs.join("\n---\n")
    };

    let grounding_instruction = if criteria.requires_grounding {
        "Base your answer ONLY on the data above. Do not make up information."
    } else {
        "You may provide general guidance based on your knowledge."
    };

    // v0.1.4: Always include system context
    let prompt = format!(
        r#"You are Anna, an AI assistant for Arch Linux systems.
This is an Arch Linux system using pacman for packages.
Do NOT suggest apt, brew, or other package managers.

Question: {}

Data collected:
{}

{}

Provide a clear, helpful answer. Be concise but complete."#,
        question, data_context, grounding_instruction
    );

    let answer = ollama::chat_with_timeout(model, &prompt, 60).await?;
    Ok(answer.trim().to_string())
}

/// Self-evaluate the answer - is it good enough?
async fn self_evaluate(
    model: &str,
    question: &str,
    answer: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
) -> Result<SelfEvaluation> {
    // Quick heuristic checks first
    if answer.len() < 20 {
        return Ok(SelfEvaluation {
            is_complete: false,
            confidence: 0.2,
            missing: Some("Answer too short".to_string()),
            suggestions: Some("Provide more detail".to_string()),
        });
    }

    // Check quality heuristics
    if !quick_quality_check(answer) {
        return Ok(SelfEvaluation {
            is_complete: false,
            confidence: 0.3,
            missing: Some("Answer quality check failed".to_string()),
            suggestions: Some("Regenerate with better grounding".to_string()),
        });
    }

    // For simple/HowTo questions, skip LLM evaluation
    if matches!(criteria.answer_type, AnswerType::Simple | AnswerType::HowTo) {
        return Ok(SelfEvaluation {
            is_complete: true,
            confidence: 0.8,
            missing: None,
            suggestions: None,
        });
    }

    // LLM self-evaluation for complex questions
    let data_summary = if state.outputs.is_empty() {
        "No data collected".to_string()
    } else {
        format!("{} command outputs collected", state.outputs.len())
    };

    let prompt = format!(
        r#"Evaluate this answer:

Question: {}
Answer: {}
Data: {}

Rate on these criteria:
1. Does it directly answer the question? (YES/NO)
2. Is it grounded in the data collected? (YES/NO/NA)
3. Is anything important missing? (describe or NONE)

Format: COMPLETE/INCOMPLETE, CONFIDENCE (0-100), MISSING: <text>"#,
        question, answer, data_summary
    );

    let response = ollama::chat_with_timeout(model, &prompt, 20).await?;
    let response = response.to_uppercase();

    // Parse response
    let is_complete = response.contains("COMPLETE") && !response.contains("INCOMPLETE");

    let confidence = if let Some(conf_match) = response
        .split_whitespace()
        .find(|w| w.parse::<f32>().is_ok())
    {
        conf_match.parse::<f32>().unwrap_or(50.0) / 100.0
    } else if is_complete {
        0.8
    } else {
        0.4
    };

    let missing = if response.contains("MISSING:") {
        response
            .split("MISSING:")
            .nth(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "NONE")
    } else {
        None
    };

    Ok(SelfEvaluation {
        is_complete,
        confidence: confidence.clamp(0.0, 1.0),
        missing: missing.clone(),
        suggestions: missing,
    })
}

/// Streaming version of the Ralph loop
/// Sends progress updates to the client in real-time
pub async fn ralph_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
) -> Result<AskResult> {
    use anna_shared::rpc::StreamingResponse;

    let criteria = determine_criteria(question);
    info!(
        "Ralph streaming: {:?}, max {} iterations",
        criteria.answer_type, criteria.max_iterations
    );

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record and send user's question
    let step = DialogueStep {
        step_type: StepType::UserQuestion,
        content: question.to_string(),
    };
    dialogue.push(step.clone());
    send_step(writer, step).await?;

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        debug!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        // Step 1: Get commands
        let commands = get_commands(model, question, &state).await?;

        if !commands.is_empty() {
            // Execute commands and stream progress
            for cmd in &commands {
                let step = DialogueStep {
                    step_type: StepType::CommandExec,
                    content: cmd.clone(),
                };
                dialogue.push(step.clone());
                send_step(writer, step).await?;

                match execute_command(cmd) {
                    Ok(output) => {
                        let clean_output = strip_ansi_codes(&output);
                        state.commands.push(cmd.clone());
                        state.outputs.push(clean_output.clone());

                        let step = DialogueStep {
                            step_type: StepType::CommandOutput,
                            content: truncate(&clean_output, 500),
                        };
                        dialogue.push(step.clone());
                        send_step(writer, step).await?;
                    }
                    Err(e) => {
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

        // Step 4: Check completion
        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            // Stream the final answer token by token
            let step = DialogueStep {
                step_type: StepType::FinalPrompt,
                content: String::new(),
            };
            send_step(writer, step).await?;

            // Stream tokens
            for token in answer.split_inclusive(' ') {
                let resp = StreamingResponse::Token {
                    token: token.to_string(),
                };
                let json = serde_json::to_string(&resp)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                writer.flush().await?;
            }

            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.clone(),
            });

            // Send done
            let result = AskResult {
                answer,
                success: true,
                iterations: iteration,
                commands_executed: state.commands,
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
            };

            let resp = StreamingResponse::Done {
                result: result.clone(),
            };
            let json = serde_json::to_string(&resp)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            writer.flush().await?;

            return Ok(result);
        }

        // Not done - prepare for next iteration
        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
    }

    // Max iterations - return best effort
    let final_answer = state.answer.unwrap_or_else(|| {
        "I couldn't fully answer your question. Please try rephrasing.".to_string()
    });

    let step = DialogueStep {
        step_type: StepType::FinalPrompt,
        content: String::new(),
    };
    send_step(writer, step).await?;

    // Stream tokens
    for token in final_answer.split_inclusive(' ') {
        let resp = anna_shared::rpc::StreamingResponse::Token {
            token: token.to_string(),
        };
        let json = serde_json::to_string(&resp)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
    }

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    });

    let result = AskResult {
        answer: final_answer,
        success: state.confidence >= 0.5,
        iterations: iteration,
        commands_executed: state.commands,
        dialogue,
        needs_clarification: state.confidence < 0.3, // v0.1.6: Lowered from 0.5 to reduce note spam
        clarification_question: state.not_done_reason,
        cached: false,
    };

    let resp = anna_shared::rpc::StreamingResponse::Done {
        result: result.clone(),
    };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;

    Ok(result)
}

/// Send a step over the streaming connection
async fn send_step<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
) -> Result<()> {
    let resp = anna_shared::rpc::StreamingResponse::Step { step };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_criteria_factual() {
        let criteria = determine_criteria("what is my kernel version?");
        assert!(matches!(criteria.answer_type, AnswerType::Factual));
        assert!(criteria.requires_grounding);
    }

    #[test]
    fn test_determine_criteria_howto() {
        let criteria = determine_criteria("how do I install neovim?");
        assert!(matches!(criteria.answer_type, AnswerType::HowTo));
        assert!(!criteria.requires_grounding);
    }

    #[test]
    fn test_determine_criteria_troubleshoot() {
        let criteria = determine_criteria("wifi not working after update");
        assert!(matches!(criteria.answer_type, AnswerType::Troubleshoot));
        assert!(criteria.requires_grounding);
    }
}
