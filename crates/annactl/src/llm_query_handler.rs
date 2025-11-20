//! LLM Query Handler - Natural language query processing
//!
//! Beta.146: Extracted from main.rs and refactored to use query_handler
//!
//! Handles one-shot natural language queries using the 3-tier architecture.

use anna_common::display::UI;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::{self, Write};

use crate::query_handler;

/// Handle a one-shot natural language query
///
/// This is used for: annactl "free storage space"
/// Uses 3-tier architecture from query_handler.
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

    // Show thinking indicator
    if ui.capabilities().use_colors() {
        print!("{} ", "anna (thinking):".bright_magenta().dimmed());
        io::stdout().flush().unwrap();
    } else {
        print!("anna (thinking): ");
        io::stdout().flush().unwrap();
    }

    // Tier 1: Try template matching first (instant, accurate)
    if let Some((template_id, params)) = query_handler::try_template_match(user_text) {
        match query_handler::execute_template(template_id, &params) {
            Ok(query_handler::QueryResult::Template { command, output, .. }) => {
                // Clear thinking line and show result
                println!("\r{}", " ".repeat(50));
                if ui.capabilities().use_colors() {
                    println!("{} {}", "anna:".bright_magenta().bold(), "Running:".white());
                } else {
                    println!("anna: Running:");
                }
                ui.info(&format!("  $ {}", command));
                println!();

                // Show command output
                for line in output.lines() {
                    ui.info(line);
                }
                println!();
                return Ok(());
            }
            Err(e) => {
                // Template execution failed, fall through to next tier
                eprintln!("Template execution failed: {}", e);
            }
            _ => {}
        }
    }

    // Tier 2: Try RecipePlanner
    let config = query_handler::get_llm_config();
    match query_handler::try_recipe_planner(user_text, &config).await {
        Ok(Some(recipe)) => {
            // Recipe generated successfully
            println!("\r{}", " ".repeat(50));
            if ui.capabilities().use_colors() {
                println!("{}", "anna:".bright_magenta().bold());
            } else {
                println!("anna:");
            }
            println!();

            display_and_execute_recipe(&recipe, &ui).await?;
            return Ok(());
        }
        Ok(None) => {
            // Recipe planning failed, fall through to LLM
            println!("\r{}", " ".repeat(50));
            if ui.capabilities().use_colors() {
                ui.warning("Could not generate safe recipe. Falling back to conversational LLM...");
            }
            println!();
        }
        Err(e) => {
            eprintln!("Recipe planning error: {}", e);
        }
    }

    // Tier 3: Generic LLM fallback (conversational)
    use anna_common::llm::{LlmClient, LlmPrompt};
    use crate::system_query::query_system_telemetry;

    // Create LLM client
    let client = match LlmClient::from_config(&config) {
        Ok(client) => client,
        Err(_) => {
            println!("\r{}", " ".repeat(50));
            ui.info("⚠ LLM not available (Ollama not running)");
            ui.info("I can still help with template-based queries!");
            println!();
            ui.info("Try: annactl \"free storage space\"");
            ui.info("Try: annactl \"check RAM\"");
            ui.info("Try: annactl \"show CPU\"");
            println!();
            return Ok(());
        }
    };

    // Build system context
    let system_context = if let Ok(telemetry) = query_system_telemetry() {
        format!(
            "System Information:\n\
             - CPU: {}\n\
             - RAM: {:.1} GB used / {:.1} GB total\n\
             - GPU: {}\n\
             - OS: Arch Linux",
            telemetry.hardware.cpu_model,
            telemetry.memory.used_mb as f64 / 1024.0,
            telemetry.memory.total_mb as f64 / 1024.0,
            telemetry.hardware.gpu_info.as_deref().unwrap_or("None"),
        )
    } else {
        "OS: Arch Linux".to_string()
    };

    // Create prompt with system context
    let system_prompt = format!(
        "{}\n\n{}",
        LlmClient::anna_system_prompt(),
        system_context
    );

    let prompt = LlmPrompt {
        system: system_prompt,
        user: user_text.to_string(),
        conversation_history: None,
    };

    // Stream response word-by-word
    let mut response_started = false;
    let mut callback = |chunk: &str| {
        if !response_started {
            // Clear thinking indicator and start response
            println!("\r{}", " ".repeat(50));
            if ui.capabilities().use_colors() {
                print!("{} ", "anna:".bright_magenta().bold());
            } else {
                print!("anna: ");
            }
            response_started = true;
        }

        // Print each chunk as it arrives
        if ui.capabilities().use_colors() {
            print!("{}", chunk.white());
        } else {
            print!("{}", chunk);
        }
        io::stdout().flush().unwrap();
    };

    match client.chat_stream(&prompt, &mut callback) {
        Ok(_) => {
            println!("\n");
        }
        Err(_) => {
            if !response_started {
                println!("\r{}", " ".repeat(50));
            }
            println!();
            ui.info("⚠ LLM request failed");
            println!();
        }
    }

    Ok(())
}

/// Display and execute a recipe
async fn display_and_execute_recipe(
    recipe: &anna_common::command_recipe::Recipe,
    ui: &UI,
) -> Result<()> {
    use anna_common::command_recipe::SafetyLevel;

    // Display recipe
    ui.section_header("🔧", "Recipe Plan");
    println!();
    ui.info(&format!("Summary: {}", recipe.summary));
    ui.info(&format!("Steps: {}", recipe.steps.len()));
    ui.info(&format!("Safety: {:?}", recipe.overall_safety));
    println!();

    // Show steps
    for (i, step) in recipe.steps.iter().enumerate() {
        ui.info(&format!("{}. {}", i + 1, step.explanation));
        ui.info(&format!("   $ {}", step.command));
    }
    println!();

    // Check if confirmation needed
    let needs_confirmation = matches!(recipe.overall_safety, SafetyLevel::NeedsConfirmation);

    if needs_confirmation {
        ui.warning("This recipe requires confirmation.");
        ui.info("Execute? (y/N): ");

        use std::io::BufRead;
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;

        if !line.trim().eq_ignore_ascii_case("y") {
            ui.info("Recipe execution cancelled.");
            return Ok(());
        }
    }

    // Execute recipe steps
    ui.info("Executing recipe...");
    println!();

    // Beta.146: Simplified execution for refactoring
    // TODO: Re-implement proper recipe execution
    for (i, step) in recipe.steps.iter().enumerate() {
        ui.info(&format!("Step {}: {}", i + 1, step.explanation));

        use std::process::Command;
        match Command::new("sh")
            .arg("-c")
            .arg(&step.command)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    ui.success(&format!("  ✓ {}", step.command));
                } else {
                    ui.error(&format!("  ✗ Command failed: {}", step.command));
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        ui.error(&format!("  Error: {}", stderr));
                    }
                }
            }
            Err(e) => {
                ui.error(&format!("  ✗ Failed to execute: {}", e));
            }
        }
    }

    ui.success("✓ Recipe execution complete!");
    println!();
    Ok(())
}
