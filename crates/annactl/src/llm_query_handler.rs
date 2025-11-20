//! LLM Query Handler - Natural language query processing
//!
//! Version 149: Now uses unified_query_handler for consistency with TUI
//!
//! Handles one-shot natural language queries.

use anna_common::display::UI;
use anna_common::llm::LlmConfig;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::{self, Write};

use crate::system_query::query_system_telemetry;
use crate::unified_query_handler::{handle_unified_query, AnswerConfidence, UnifiedQueryResult};

/// Handle a one-shot natural language query
///
/// This is used for: annactl "free storage space"
/// Version 149: Uses unified handler for consistency with TUI.
pub async fn handle_one_shot_query(user_text: &str) -> Result<()> {
    let ui = UI::auto();

    // Show user's question with nice formatting
    println!();
    if ui.capabilities().use_colors() {
        println!("{} {}", "you:".bright_cyan().bold(), user_text.white());
    } else {
        println!("you: {}", user_text);
    }
    println!();

    // Show thinking indicator with animation
    show_thinking_animation(&ui, "Thinking");

    // Get telemetry
    let telemetry = query_system_telemetry()?;

    // Get LLM config
    let config = get_llm_config();

    // Use unified query handler
    match handle_unified_query(user_text, &telemetry, &config).await {
        Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        }) => {
            clear_thinking_line();
            println!(
                "{} {} {}",
                "anna:".bright_magenta().bold(),
                "Using deterministic recipe:".white(),
                recipe_name.bright_green()
            );
            println!();
            display_action_plan(&action_plan, &ui);
        }
        Ok(UnifiedQueryResult::Template {
            command, output, ..
        }) => {
            clear_thinking_line();
            println!("{} {}", "anna:".bright_magenta().bold(), "Running:".white());
            ui.info(&format!("  $ {}", command));
            println!();
            for line in output.lines() {
                ui.info(line);
            }
            println!();
        }
        Ok(UnifiedQueryResult::ActionPlan {
            action_plan,
            raw_json: _,
        }) => {
            clear_thinking_line();
            println!("{}", "anna:".bright_magenta().bold());
            println!();
            display_action_plan(&action_plan, &ui);
        }
        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence,
            sources,
        }) => {
            clear_thinking_line();
            println!("{}", "anna:".bright_magenta().bold());
            println!();

            // Display answer
            for line in answer.lines() {
                ui.info(line);
            }
            println!();

            // Show confidence and sources (subtle)
            if ui.capabilities().use_colors() {
                println!(
                    "{}",
                    format!("🔍 Confidence: {:?} | Sources: {}", confidence, sources.join(", "))
                        .dimmed()
                );
            }
            println!();
        }
        Err(e) => {
            clear_thinking_line();
            ui.error(&format!("Query failed: {}", e));
            println!();
        }
    }

    Ok(())
}

/// Show thinking animation (Version 149: Added for CLI)
fn show_thinking_animation(ui: &UI, _message: &str) {
    if ui.capabilities().use_colors() {
        print!("{} ", "anna (thinking):".bright_magenta().dimmed());
        io::stdout().flush().unwrap();
    } else {
        print!("anna (thinking): ");
        io::stdout().flush().unwrap();
    }
}

/// Clear thinking line
fn clear_thinking_line() {
    print!("\r{}\r", " ".repeat(80));
    io::stdout().flush().unwrap();
}

/// Display ActionPlan
fn display_action_plan(plan: &anna_common::action_plan_v3::ActionPlan, ui: &UI) {
    // Show analysis
    ui.section_header("📋", "Analysis");
    println!("{}\n", plan.analysis);

    // Show goals
    ui.section_header("🎯", "Goals");
    for (i, goal) in plan.goals.iter().enumerate() {
        println!("  {}. {}", i + 1, goal);
    }
    println!();

    // Show command plan
    if !plan.command_plan.is_empty() {
        ui.section_header("⚡", "Command Plan");
        for (i, step) in plan.command_plan.iter().enumerate() {
            println!("  {}. {} [Risk: {:?}]", i + 1, step.description, step.risk_level);
            println!("     $ {}", step.command);
        }
        println!();
    }

    // Show notes
    if !plan.notes_for_user.is_empty() {
        ui.section_header("ℹ️", "Notes");
        println!("{}\n", plan.notes_for_user);
    }
}

/// Stream LLM response
async fn stream_llm_response(prompt: &str, config: &LlmConfig, ui: &UI) -> Result<()> {
    use anna_common::llm::{LlmClient, LlmPrompt};

    let client = match LlmClient::from_config(config) {
        Ok(client) => client,
        Err(_) => {
            clear_thinking_line();
            ui.info("⚠ LLM not available (Ollama not running)");
            return Ok(());
        }
    };

    let llm_prompt = LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt.to_string(),
        conversation_history: None,
    };

    let mut response_started = false;
    let mut callback = |chunk: &str| {
        if !response_started {
            clear_thinking_line();
            if ui.capabilities().use_colors() {
                print!("{} ", "anna:".bright_magenta().bold());
            } else {
                print!("anna: ");
            }
            response_started = true;
        }

        if ui.capabilities().use_colors() {
            print!("{}", chunk.white());
        } else {
            print!("{}", chunk);
        }
        io::stdout().flush().unwrap();
    };

    match client.chat_stream(&llm_prompt, &mut callback) {
        Ok(_) => println!("\n"),
        Err(_) => {
            if !response_started {
                clear_thinking_line();
            }
            println!();
            ui.info("⚠ LLM request failed");
        }
    }

    Ok(())
}

/// Get LLM config
fn get_llm_config() -> LlmConfig {
    use std::process::Command;

    let model_name = match Command::new("ollama").arg("list").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .nth(1)
                .and_then(|line| line.split_whitespace().next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "llama3.1:8b".to_string())
        }
        _ => "llama3.1:8b".to_string(),
    };

    LlmConfig::local("http://127.0.0.1:11434/v1", &model_name)
}
