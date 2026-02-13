//! Answer generation and self-evaluation for the Ralph loop.

use anyhow::Result;

use crate::ollama;
use super::criteria::{AnswerType, CompletionCriteria, IterationState, SelfEvaluation, quick_quality_check};

/// v0.3.111: Check learned recipes for command patterns.
/// Returns commands if a high-confidence recipe match is found.
pub fn check_recipes_for_commands(question: &str) -> Option<Vec<String>> {
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

/// Generate an answer based on collected data
pub async fn generate_answer(
    model: &str,
    question: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
    wiki_research: Option<&str>,
) -> Result<String> {
    let data_context = if state.outputs.is_empty() {
        "No command output available.".to_string()
    } else {
        state.outputs.join("\n---\n")
    };

    let grounding_instruction = if criteria.requires_grounding {
        "Base your answer ONLY on the data above and wiki documentation. Do not make up information."
    } else {
        "You may provide general guidance based on your knowledge and documentation."
    };

    // v0.3.110: Include live system state for context
    let live_state = anna_shared::live_state::LiveState::capture();
    let system_context = if live_state.is_stressed() {
        format!("\nCurrent system state (STRESSED): {}", live_state.summary())
    } else {
        format!("\nCurrent system state: {}", live_state.summary())
    };

    // v0.3.131: Include wiki research if available
    let wiki_context = if let Some(research) = wiki_research {
        if !research.is_empty() {
            format!("\n\n{}", research)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // v0.3.186: Include man pages and --help output for command-related questions
    let docs_context = {
        let citations = anna_shared::docs::search_docs(question);
        if citations.is_empty() {
            String::new()
        } else {
            let mut ctx = "\n\nLocal Documentation:".to_string();
            for citation in citations.iter().take(2) {
                ctx.push_str(&format!("\n[{}]\n{}", citation.format_short(), citation.excerpt));
            }
            ctx
        }
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

The user asked: "{question}"

You ran investigation commands and collected this output:
{data_context}
{wiki_context}{docs_context}{web_context}

{grounding_instruction}

Your task: Answer the user's question directly using the output above.
The data was collected by YOU running commands — do not say the user provided it.
If the question asks for a report or summary, produce one from the data.
Be concise but complete."#,
        question = question,
        data_context = data_context,
        grounding_instruction = grounding_instruction,
        system_context = system_context,
        wiki_context = wiki_context,
        docs_context = docs_context,
        web_context = web_context
    );

    let answer = ollama::chat_with_timeout(model, &prompt, 60).await?;
    Ok(answer.trim().to_string())
}

/// Check if question is about an error or problem.
pub fn is_error_or_problem(question: &str) -> bool {
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
pub fn contains_error_output(outputs: &[String]) -> bool {
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
    let response_upper = response.to_uppercase();

    // Parse COMPLETE/INCOMPLETE from the first non-empty line only
    // (avoids false match on format echo "COMPLETE/INCOMPLETE" which contains both words).
    let first_line = response_upper.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let is_complete = first_line.contains("COMPLETE") && !first_line.contains("INCOMPLETE");

    // Parse confidence from the second comma-separated field, not the first number anywhere.
    // Avoids grabbing counts/years from the MISSING text (e.g., "MISSING: only 1 of 3 partitions").
    let confidence = response_upper
        .split(',')
        .nth(1) // second field: "CONFIDENCE 85" or " CONFIDENCE 85"
        .and_then(|field| {
            field.split_whitespace()
                .find_map(|w| w.parse::<f32>().ok())
        })
        .map(|v| v / 100.0)
        .unwrap_or(if is_complete { 0.8 } else { 0.4 });
    let response_upper = response_upper; // rebind for use below

    let missing = if response_upper.contains("MISSING:") {
        response_upper
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
