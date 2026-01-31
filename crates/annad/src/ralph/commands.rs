//! Command execution and answer generation for the Ralph loop.

use anyhow::Result;

use crate::ollama;
use super::criteria::{AnswerType, CompletionCriteria, IterationState, SelfEvaluation, quick_quality_check};

/// v0.3.111: Check learned recipes for command patterns.
/// Returns commands if a high-confidence recipe match is found.
fn check_recipes_for_commands(question: &str) -> Option<Vec<String>> {
    // Load recipe book
    let book = anna_shared::recipe::RecipeBook::load().ok()?;

    // Get system context for matching
    let system_info = anna_shared::profile::SystemInfo::default();

    // Find matching recipes
    let matches = book.find_matches(question, &system_info);

    // Only use recipes with good success history
    if let Some(recipe) = matches.first() {
        // Skip recipes with no success history (unproven)
        if recipe.success_count == 0 && !matches!(recipe.source, anna_shared::recipe::RecipeSource::BuiltIn) {
            return None;
        }

        // Extract non-modifying commands for investigation
        let commands: Vec<String> = recipe.commands.iter()
            .filter(|c| !c.modifies_system)
            .map(|c| c.command.clone())
            .collect();

        if !commands.is_empty() {
            tracing::debug!(
                "Recipe '{}' matched with {} commands (success_count={})",
                recipe.name, commands.len(), recipe.success_count
            );
            return Some(commands);
        }
    }

    None
}

/// Result of asking the LLM what to do next.
pub enum NextAction {
    /// Run these investigation commands.
    Commands(Vec<String>),
    /// No commands needed (already answered or how-to).
    None,
    /// This is a config request - generate an ActionPlan via LLM.
    Config,
}

/// Get commands to run for answering the question.
/// May also detect config requests and return NextAction::Config.
/// v0.3.111: Checks learned recipes first for faster response.
pub async fn get_next_action(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<NextAction> {
    use crate::llm_core::prompts::system_context;

    // v0.3.111: Check recipes first for learned patterns
    if state.commands.is_empty() {
        if let Some(commands) = check_recipes_for_commands(question) {
            tracing::info!("Using {} commands from learned recipe", commands.len());
            return Ok(NextAction::Commands(commands));
        }
    }

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
        r#"{context}

Question: "{question}"{output_context}{feedback_context}

Determine what to do. Output EXACTLY ONE of these formats:

FORMAT 1 - Run investigation commands (to gather info):
COMMANDS:
<command1>
<command2>

FORMAT 2 - This is a system configuration request (change settings, enable/disable, install, etc.):
CONFIG

FORMAT 3 - You can answer from knowledge alone (how-to, explanations):
NONE

FORMAT 4 - Data already collected is sufficient:
DONE

COMMAND REFERENCE:
SYSTEM: uname -r, uptime -p, hostnamectl
HARDWARE: lscpu | head -20, free -h, lsusb, lspci | head -20
DESKTOP: echo $XDG_CURRENT_DESKTOP, echo $XDG_SESSION_TYPE
STORAGE: df -h, lsblk, findmnt / -o OPTIONS, swapon --show
NETWORK: ip -4 addr show, cat /etc/resolv.conf, ip route | grep default, ss -tlnp | head -15
SERVICES: systemctl --failed, systemctl list-units --type=service --state=running | head -20
PACKAGES: pacman -Q | wc -l, pacman -Qe | head -30
LOGS: journalctl -p err -b --no-pager | head -30

RULES:
- For info/diagnostic questions: use COMMANDS format
- For "set", "change", "disable", "enable", "install", "configure", "prevent" requests: use CONFIG
- For "how do I", "what is", "explain" questions: use NONE
- Output ONLY the format above, no explanations

Output now:"#,
        context = system_context(),
        question = question,
        output_context = output_context,
        feedback_context = feedback_context,
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    let response = response.trim();
    let response_upper = response.to_uppercase();

    if response_upper.starts_with("CONFIG") || response_upper == "CONFIG" {
        return Ok(NextAction::Config);
    }

    if response_upper == "NONE" || response_upper == "DONE" || response.is_empty() {
        return Ok(NextAction::None);
    }

    // Parse commands (strip "COMMANDS:" prefix if present)
    let cmd_text = if response_upper.starts_with("COMMANDS:") {
        &response[9..]
    } else {
        response
    };

    let commands: Vec<String> = cmd_text
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            if l.is_empty() || l.starts_with('#') {
                return false;
            }
            let upper = l.to_uppercase();
            if upper == "DONE" || upper == "NONE" || upper.starts_with("DONE:")
                || upper == "CONFIG" || upper == "COMMANDS:" {
                return false;
            }
            true
        })
        .map(|l| l.to_string())
        .take(5)
        .collect();

    if commands.is_empty() {
        Ok(NextAction::None)
    } else {
        Ok(NextAction::Commands(commands))
    }
}

/// Backwards-compatible wrapper for non-streaming path.
pub async fn get_commands(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<Vec<String>> {
    match get_next_action(model, question, state).await? {
        NextAction::Commands(cmds) => Ok(cmds),
        NextAction::None | NextAction::Config => Ok(Vec::new()),
    }
}

/// Generate an answer based on collected data
pub async fn generate_answer(
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

    // v0.3.110: Include live system state for context
    let live_state = anna_shared::live_state::LiveState::capture();
    let system_context = if live_state.is_stressed() {
        format!("\nCurrent system state (STRESSED): {}", live_state.summary())
    } else {
        format!("\nCurrent system state: {}", live_state.summary())
    };

    // v0.3.112: Search web for error/problem solutions when needed
    let web_context = if is_error_or_problem(question) || contains_error_output(&state.outputs) {
        match anna_shared::web_search::search_for_solution(question, 3).await {
            Ok(results) if !results.is_empty() => {
                tracing::debug!("Web search found {} results", results.len());
                format!("\n\n{}", anna_shared::web_search::format_results_for_context(&results))
            }
            _ => String::new()
        }
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are Anna, an AI assistant for Arch Linux systems.
This is an Arch Linux system using pacman for packages.
Do NOT suggest apt, brew, or other package managers.
{system_context}

Question: {}

Data collected:
{}
{web_context}

{}

Provide a clear, helpful answer. Be concise but complete."#,
        question, data_context, grounding_instruction,
        system_context = system_context,
        web_context = web_context
    );

    let answer = ollama::chat_with_timeout(model, &prompt, 60).await?;
    Ok(answer.trim().to_string())
}

/// Check if question is about an error or problem.
fn is_error_or_problem(question: &str) -> bool {
    let q_lower = question.to_lowercase();
    q_lower.contains("error") ||
    q_lower.contains("fail") ||
    q_lower.contains("not working") ||
    q_lower.contains("broken") ||
    q_lower.contains("can't") ||
    q_lower.contains("cannot") ||
    q_lower.contains("won't") ||
    q_lower.contains("doesn't work") ||
    q_lower.contains("problem") ||
    q_lower.contains("issue")
}

/// Check if command outputs contain error indicators.
fn contains_error_output(outputs: &[String]) -> bool {
    outputs.iter().any(|o| {
        let o_lower = o.to_lowercase();
        o_lower.contains("error:") ||
        o_lower.contains("failed") ||
        o_lower.contains("permission denied") ||
        o_lower.contains("no such file") ||
        o_lower.contains("command not found")
    })
}

/// Self-evaluate the answer - is it good enough?
pub async fn self_evaluate(
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
