//! LLM Query Handler - Natural language query processing
//!
//! Beta.200: Unified workflow for one-shot queries
//!
//! Implements telemetry-first architecture:
//! 1. Detect intent (informational vs actionable)
//! 2. Match to deterministic recipe (if actionable)
//! 3. Generate answer based on real telemetry
//!
//! Uses unified_query_handler for consistency with TUI mode.

use anna_common::display::UI;
use anna_common::llm::LlmConfig;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::time::Duration;

use crate::llm_integration::fetch_telemetry_snapshot;
use crate::output::normalize_for_cli;
use crate::startup::welcome::{generate_welcome_report, load_last_session, save_session_metadata};
use crate::system_query::query_system_telemetry;
use crate::unified_query_handler::{handle_unified_query, AnswerConfidence, UnifiedQueryResult};

/// Handle a one-shot natural language query
///
/// This is used for: annactl "free storage space"
/// Version 149: Uses unified handler for consistency with TUI.
/// Beta.228: Added comprehensive logging
pub async fn handle_one_shot_query(user_text: &str) -> Result<()> {
    let start_time = std::time::Instant::now();

    let ui = UI::auto();

    // Show user's question with nice formatting
    println!();
    if ui.capabilities().use_colors() {
        println!("{} {}", "you:".bright_cyan().bold(), user_text.white());
    } else {
        println!("you: {}", user_text);
    }
    println!();

    // Create spinner for thinking animation (Beta.202: Professional animation)
    let spinner = create_thinking_spinner(&ui);

    // Get telemetry
    let telemetry_start = std::time::Instant::now();
    let telemetry = query_system_telemetry()?;

    // Get LLM config
    let config = get_llm_config();

    // Beta.229: Stop spinner before unified handler to prevent corruption during streaming
    spinner.finish_and_clear();

    // Use unified query handler
    let handler_start = std::time::Instant::now();
    match handle_unified_query(user_text, &telemetry, &config).await {
        Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        }) => {
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
            println!("{}", "anna:".bright_magenta().bold());
            println!();
            display_action_plan(&action_plan, &ui);
        }
        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence,
            sources,
        }) => {

            // Beta.229: DISABLED - Welcome report adds 19s delay to one-shot queries
            // The fetch_telemetry_snapshot() and generate_welcome_report() are extremely slow
            // Re-enable in Beta.230+ with performance optimization or async background task

            // Beta.229: Answer already streamed to stdout during LLM call
            // Don't print it again! Just show metadata
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
            spinner.finish_and_clear();
            ui.error(&format!("Query failed: {}", e));
            println!();
        }
    }

    Ok(())
}

/// Create thinking spinner (Beta.202: Professional animation)
fn create_thinking_spinner(ui: &UI) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();

    if ui.capabilities().use_colors() {
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.magenta} {msg}")
                .unwrap()
        );
        spinner.set_message("anna (thinking)...".to_string());
    } else {
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["-", "\\", "|", "/"])
                .template("{spinner} {msg}")
                .unwrap()
        );
        spinner.set_message("anna (thinking)...".to_string());
    }

    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
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

/// Stream LLM response (Beta.202: Updated for spinner)
async fn stream_llm_response(prompt: &str, config: &LlmConfig, ui: &UI, spinner: &ProgressBar) -> Result<()> {
    use anna_common::llm::{LlmClient, LlmPrompt};

    let client = match LlmClient::from_config(config) {
        Ok(client) => client,
        Err(_) => {
            spinner.finish_and_clear();
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
            spinner.finish_and_clear();
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
                spinner.finish_and_clear();
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
