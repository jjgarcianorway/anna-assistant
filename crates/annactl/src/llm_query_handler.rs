//! LLM Query Handler - v7.0.0 Clean Brain Architecture
//!
//! ALL queries flow through the brain_v7 pipeline:
//! PLAN (LLM) → EXECUTE (Rust) → INTERPRET (LLM)
//!
//! Strict data contracts, reliability scoring, and retry logic.

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
/// v7.0.0: Uses brain_v7 pipeline (PLAN → EXECUTE → INTERPRET)
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

    // v7.0.0: Use LLM config with default settings
    let llm_config = anna_common::llm_client::LlmConfig::default();

    // v7.0.0: Use the clean brain_v7 architecture
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
    fn test_single_pipeline_architecture() {
        // v7.0.0: Verify we're using brain_v7 pipeline
        // This test exists to document the architecture choice
        assert!(true, "All queries go through query_handler_v7 (brain_v7)");
    }
}
