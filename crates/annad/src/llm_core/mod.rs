//! LLM-Only Core Loop - No pattern matching, pure intelligence.
//!
//! Architecture:
//! 1. UNDERSTAND - LLM parses intent and what info is needed
//! 2. INVESTIGATE - LLM decides commands, executes in stages
//! 3. ANALYZE - LLM correlates findings, identifies issues
//! 4. RESPOND - Grounded answer or smart fix suggestion
//!
//! Key principles:
//! - LLM always decides, no hardcoded patterns
//! - Multi-stage investigation (overview -> deep dive)
//! - Fixes suggested based on actual findings, not keywords
//! - All answers grounded in command output

pub mod commands;
pub mod evidence;
pub mod investigate;
pub mod prompts;
pub mod streaming;
pub mod types;

pub use streaming::execute_question_streaming_llm;
pub use types::{Finding, InvestigationState, NextStep, Understanding, VerificationResult};

use anna_shared::rpc::AskResult;
use anyhow::Result;
use tracing::{debug, info};

use crate::core_loop::command::execute_command;
use crate::ollama::chat_with_timeout;

use commands::is_valid_command;
use evidence::verify_answer;

/// Maximum investigation iterations (keep low to avoid timeouts)
pub(crate) const MAX_ITERATIONS: u8 = 3;
/// LLM timeout in seconds
const LLM_TIMEOUT_SECS: u64 = 60;

/// Main entry point - execute a question using pure LLM intelligence
pub async fn execute_question_llm(model: &str, question: &str) -> Result<AskResult> {
    info!("LLM Core: Processing question: {}", question);

    let mut state = InvestigationState::default();
    let dialogue = Vec::new();

    // PHASE 1: UNDERSTAND - What is the user asking?
    let understanding = understand_question(model, question).await?;
    debug!("Understanding: {:?}", understanding);

    // Check if out of scope
    if let Some(reason) = &understanding.out_of_scope_reason {
        return Ok(AskResult {
            answer: format!("I'm Anna, your Linux assistant. {}", reason),
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue,
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
        });
    }

    // PHASE 2: INVESTIGATE - Gather information
    loop {
        state.iteration += 1;
        if state.iteration > MAX_ITERATIONS {
            info!("Max iterations reached");
            break;
        }

        // Ask LLM what to do next
        let next = decide_next_step(model, question, &state).await?;

        match next {
            NextStep::Investigate(commands) => {
                info!("Iteration {}: Running {} commands", state.iteration, commands.len());
                execute_investigation(&mut state, commands).await?;
            }
            NextStep::Answer => {
                info!("LLM decided: enough info to answer");
                break;
            }
            NextStep::SuggestFix { problem, fix_command, explanation } => {
                return finish_with_suggested_fix(
                    &state, &dialogue, question, &problem, &fix_command, &explanation
                );
            }
            NextStep::OutOfScope(reason) => {
                return Ok(AskResult {
                    answer: format!("I'm Anna, your Linux assistant. {}", reason),
                    success: true,
                    iterations: state.iteration as u32,
                    commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                });
            }
        }
    }

    // PHASE 3: RESPOND - Generate grounded answer
    let raw_answer = generate_answer(model, question, &state).await?;

    // v0.3.25: Verify through ClaimGate and append evidence line
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

    // Append evidence line if we have any findings
    let answer = if verification.evidence_line.is_empty() {
        verification.answer
    } else {
        format!("{}\n\n{}", verification.answer, verification.evidence_line)
    };

    Ok(AskResult {
        answer,
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    })
}

/// Execute investigation commands and collect findings
async fn execute_investigation(state: &mut InvestigationState, commands: Vec<String>) -> Result<()> {
    for cmd in commands {
        let cmd_clone = cmd.clone();
        let result = tokio::task::spawn_blocking(move || {
            execute_command(&cmd_clone)
        }).await?;

        let (output, success) = match result {
            Ok(out) => (out, true),
            Err(e) => (format!("Error: {}", e), false),
        };

        state.findings.push(Finding {
            command: cmd,
            output,
            success,
        });
    }
    Ok(())
}

/// Finish with a suggested fix response
fn finish_with_suggested_fix(
    state: &InvestigationState,
    dialogue: &[anna_shared::rpc::DialogueStep],
    question: &str,
    problem: &str,
    fix_command: &str,
    explanation: &str,
) -> Result<AskResult> {
    info!("LLM found problem, suggesting fix");
    let raw_answer = format!(
        "I found the issue: {}\n\n\
         I can fix this by running:\n  {}\n\n\
         {}\n\n\
         Would you like me to do this? (yes/no)",
        problem, fix_command, explanation
    );

    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

    let answer = if verification.evidence_line.is_empty() {
        verification.answer
    } else {
        format!("{}\n\n{}", verification.answer, verification.evidence_line)
    };

    Ok(AskResult {
        answer,
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue: dialogue.to_vec(),
        needs_clarification: true,
        clarification_question: Some("Confirm fix?".to_string()),
        cached: false,
        citations: vec![],
    })
}

/// Use LLM to understand the question
pub(crate) async fn understand_question(model: &str, question: &str) -> Result<Understanding> {
    let prompt = prompts::understanding_prompt(question);
    let response = chat_with_timeout(model, &prompt, LLM_TIMEOUT_SECS).await?;

    // Parse LLM response
    let mut intent = "factual".to_string();
    let mut info_needed = Vec::new();
    let mut out_of_scope_reason = None;

    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("INTENT:") {
            intent = line.trim_start_matches("INTENT:").trim().to_lowercase();
        } else if line.starts_with("NEED:") {
            info_needed.push(line.trim_start_matches("NEED:").trim().to_string());
        } else if line.starts_with("OUT_OF_SCOPE:") {
            out_of_scope_reason = Some(line.trim_start_matches("OUT_OF_SCOPE:").trim().to_string());
        }
    }

    Ok(Understanding { intent, info_needed, out_of_scope_reason })
}

/// Use LLM to decide what to do next
pub(crate) async fn decide_next_step(
    model: &str,
    question: &str,
    state: &InvestigationState,
) -> Result<NextStep> {
    let prompt = prompts::next_step_prompt(question, state);
    let response = chat_with_timeout(model, &prompt, LLM_TIMEOUT_SECS).await?;
    let response = response.trim();

    // Parse LLM decision
    if response.starts_with("COMMANDS:") {
        let commands: Vec<String> = response
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| l.trim().to_string())
            .filter(|cmd| is_valid_command(cmd))
            .take(3)
            .collect();

        if commands.is_empty() {
            return Ok(NextStep::Answer);
        }
        return Ok(NextStep::Investigate(commands));
    }

    if response.starts_with("ANSWER") {
        return Ok(NextStep::Answer);
    }

    if response.starts_with("FIX:") {
        return parse_fix_response(response);
    }

    if response.starts_with("OUT_OF_SCOPE:") {
        let reason = response.trim_start_matches("OUT_OF_SCOPE:").trim().to_string();
        return Ok(NextStep::OutOfScope(reason));
    }

    // Default: try to extract commands from response
    let commands: Vec<String> = response
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.is_empty() && !l.starts_with('#') && !l.contains(':')
        })
        .map(|l| l.trim().to_string())
        .filter(|cmd| is_valid_command(cmd))
        .take(2)
        .collect();

    if commands.is_empty() {
        Ok(NextStep::Answer)
    } else {
        Ok(NextStep::Investigate(commands))
    }
}

/// Parse FIX response from LLM
fn parse_fix_response(response: &str) -> Result<NextStep> {
    let mut problem = String::new();
    let mut fix_command = String::new();
    let mut explanation = String::new();

    for line in response.lines() {
        if line.starts_with("FIX:") {
            fix_command = line.trim_start_matches("FIX:").trim().to_string();
        } else if line.starts_with("PROBLEM:") {
            problem = line.trim_start_matches("PROBLEM:").trim().to_string();
        } else if line.starts_with("EXPLAIN:") {
            explanation = line.trim_start_matches("EXPLAIN:").trim().to_string();
        }
    }

    if !fix_command.is_empty() && !problem.is_empty() {
        Ok(NextStep::SuggestFix { problem, fix_command, explanation })
    } else {
        Ok(NextStep::Answer)
    }
}

/// Use LLM to generate final answer based on findings
pub(crate) async fn generate_answer(
    model: &str,
    question: &str,
    state: &InvestigationState,
) -> Result<String> {
    let prompt = prompts::answer_prompt(question, state);
    let response = chat_with_timeout(model, &prompt, LLM_TIMEOUT_SECS).await?;
    Ok(response.trim().to_string())
}
