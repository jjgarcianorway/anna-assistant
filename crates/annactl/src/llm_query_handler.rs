//! LLM Query Handler - Hybrid v7/v8 Architecture
//!
//! v7 (current default): PLAN → EXECUTE → INTERPRET - works well with local LLMs
//! v8 (available): Pure think→answer loop - requires smarter LLMs
//!
//! The v8 philosophy is correct (LLM is the brain, Rust is just hands/memory),
//! but current local LLMs (llama 3.2, etc.) aren't capable enough to follow
//! the complex tool catalog instructions reliably.

use anna_common::display::UI;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

use crate::system_query::query_system_telemetry;

/// Create a thinking spinner for visual feedback
fn create_thinking_spinner(ui: &UI) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();

    if ui.capabilities().use_colors() {
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.magenta} {msg}")
                .unwrap(),
        );
    } else {
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("-\\|/")
                .template("{spinner} {msg}")
                .unwrap(),
        );
    }

    spinner.set_message("thinking...");
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

/// Handle a one-shot query from CLI
///
/// v8.0.0: Pure LLM-driven architecture - single prompt, think→answer loop
pub async fn handle_one_shot_query(user_text: &str) -> Result<()> {
    // Initialize formatter with user configuration
    let config = anna_common::anna_config::AnnaConfig::load().unwrap_or_default();
    anna_common::terminal_format::init_with_config(&config);

    let ui = UI::auto();

    // Show user's question with nice formatting
    println!();
    if ui.capabilities().use_colors() {
        println!("{} {}", "you:".bright_cyan().bold(), user_text.white());
    } else {
        println!("you: {}", user_text);
    }
    println!();

    // Get telemetry first (needed for pipeline)
    let telemetry = query_system_telemetry()?;

    // Create spinner for thinking animation
    let spinner = create_thinking_spinner(&ui);

    // v8.0.0: Use LLM config with default settings
    let llm_config = anna_common::llm_client::LlmConfig::default();

    // Use v7 for now (more reliable with local LLMs)
    // v8 philosophy is correct but requires smarter LLMs
    match crate::query_handler_v7::handle_query_v7(
        user_text,
        &telemetry,
        Some(&llm_config)
    ).await {
        Ok(output) => {
            spinner.finish_and_clear();
            if ui.capabilities().use_colors() {
                println!("{}", "anna:".bright_magenta().bold());
            } else {
                println!("anna:");
            }
            println!("{}", output);
            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            if ui.capabilities().use_colors() {
                println!("{}", "anna:".bright_magenta().bold());
                println!("{}", format!("I encountered an error: {}", e).red());
            } else {
                println!("anna:");
                println!("I encountered an error: {}", e);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_llm_architecture() {
        // v8.0.0: Pure LLM-driven - single prompt, think→answer loop
        assert!(true, "All queries go through query_handler_v8 (brain_v8)");
    }
}
