//! Display utilities for CLI output - colors, step printing, status display.

use anna_shared::rpc::{AskResult, StepType};
use std::io::{self, Write};

// Color constants
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";

/// Print colored text (no newline)
pub fn print_colored(text: &str, color: &str) {
    print!("{}{}\x1b[0m", color, text);
}

/// Print colored text with newline
pub fn println_colored(text: &str, color: &str) {
    println!("{}{}\x1b[0m", color, text);
}

/// Print the greeting
pub fn print_greeting() {
    println!();
    println_colored("Anna - Arch Linux Assistant", BOLD);
    println_colored("Ask questions about your system in plain English.", DIM);
    println_colored("Type 'quit' or Ctrl-D to exit.", DIM);
    println!();
}

/// Print status
pub async fn print_status() {
    match crate::rpc::get_status().await {
        Ok(status) => {
            let state_color = match status.state {
                anna_shared::status::DaemonState::Ready => GREEN,
                anna_shared::status::DaemonState::Starting => YELLOW,
                anna_shared::status::DaemonState::Error => RED,
            };
            print!("Status: ");
            println_colored(&status.state.to_string(), state_color);
            println!("Version: {}", status.version);
            print!("Ollama: ");
            if status.ollama_running {
                println_colored("running", GREEN);
            } else {
                println_colored("not running", RED);
            }
            if let Some(model) = &status.model {
                println!("Model: {}", model);
            }
            if let Some(gpu) = &status.gpu {
                print!("GPU: ");
                println_colored(gpu, CYAN);
                if let Some(vram) = status.vram_mb {
                    println!("VRAM: {} MB", vram);
                }
            }
            // v0.0.924: Memory health
            print!("Memory: ");
            if status.memory_experiences == 0 {
                println_colored("empty", DIM);
            } else {
                print_colored(&format!("{} experiences", status.memory_experiences), GREEN);
                if !status.memory_health_issues.is_empty() {
                    print_colored(" (", DIM);
                    print_colored(&format!("{} issues", status.memory_health_issues.len()), YELLOW);
                    print_colored(")", DIM);
                }
                println!();
            }
            // Show health issues if any
            for issue in &status.memory_health_issues {
                print_colored("  ⚠ ", YELLOW);
                println!("{}", issue);
            }
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
}

/// Internal step printer with option to force FinalAnswer content
fn print_step_internal(step: &anna_shared::rpc::DialogueStep, force_final_answer: bool) {
    match step.step_type {
        StepType::UserQuestion => {
            print_colored("USER: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::AnnaToLlm => {
            print_colored("ANNA → LLM: ", YELLOW);
            println!("(command selection prompt)");
            print_box(&step.content);
            println!();
        }
        StepType::LlmCommands => {
            print_colored("LLM → ANNA: ", YELLOW);
            if step.content == "NONE" || step.content == "DONE" {
                println_colored(&step.content, DIM);
            } else {
                println!("commands to run:");
                print_commands(&step.content);
            }
            println!();
        }
        StepType::CommandExec => {
            print_colored("EXEC: ", GREEN);
            println!("{}", step.content);
        }
        StepType::CommandOutput => {
            print_colored("OUTPUT: ", DIM);
            println!("{}", step.content);
            println!();
        }
        StepType::ValidationPrompt => {
            print_colored("ANNA → LLM: ", YELLOW);
            println!("(validation prompt)");
            print_box(&step.content);
            println!();
        }
        StepType::ValidationResponse => {
            print_colored("LLM → ANNA: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::FinalPrompt => {
            print_colored("ANNA → LLM: ", YELLOW);
            println!("(final answer prompt)");
            print_box(&step.content);
            println!();
        }
        StepType::FinalAnswer => {
            // Streaming mode: content is empty (streamed token by token)
            // Dialogue mode: content has full answer
            if !step.content.is_empty() || force_final_answer {
                println_colored("═══════════════════════════════════════", DIM);
                print_colored("ANSWER: ", GREEN);
                if force_final_answer {
                    println!();
                }
                println_colored(&step.content, GREEN);
                println_colored("═══════════════════════════════════════", DIM);
            }
        }
        StepType::WikiSearch => {
            print_colored("ANNA → WIKI: ", MAGENTA);
            println!("searching Arch Wiki...");
            println_colored(&format!("  query: {}", step.content), DIM);
            println!();
        }
        StepType::WikiResults => {
            print_colored("WIKI → ANNA: ", MAGENTA);
            println!("found articles:");
            for line in step.content.lines() {
                println_colored(&format!("  • {}", line), DIM);
            }
            println!();
        }
        StepType::WikiCommands => {
            print_colored("WIKI: ", MAGENTA);
            println!("extracted commands:");
            print_commands(&step.content);
            println!();
        }
        StepType::ClarificationQuestion => {
            print_colored("ANNA → USER: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::ClarificationResponse => {
            print_colored("USER → ANNA: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::IntentClassifying => {
            print_colored("ANNA: ", BLUE);
            println!("understanding question...");
        }
        StepType::IntentResult => {
            print_colored("  intent: ", DIM);
            println!("{}", step.content);
        }
        StepType::SubQuestion => {
            println!();
            print_colored("─── ", DIM);
            print_colored(&step.content, YELLOW);
            println!();
        }
        StepType::SubQuestionResult => {
            print_colored("  → ", GREEN);
            println!("{}", step.content);
        }
        StepType::UnderstandingCheck => {
            print_colored("ANNA: ", CYAN);
            println!("{}", step.content);
        }
        StepType::ConfirmationRequest => {
            println!();
            print_colored("ANNA → USER: ", YELLOW);
            println!("Please confirm:");
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::MissingInfo => {
            print_colored("ANNA: ", RED);
            println!("Missing information:");
            for line in step.content.lines() {
                println!("  - {}", line);
            }
        }
        StepType::SystemAlert => {
            println!();
            println_colored("╔══════════════════════════════════════════════╗", YELLOW);
            println_colored("║           SYSTEM ALERT                       ║", YELLOW);
            println_colored("╚══════════════════════════════════════════════╝", YELLOW);
            for line in step.content.lines() {
                print_colored("  ", YELLOW);
                println!("{}", line);
            }
            println!();
        }
        StepType::LlmError => {
            print_colored("LLM ERROR: ", RED);
            if let Ok(ctx) =
                serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
            {
                println!("{} ({})", ctx.message, format!("{:?}", ctx.error_type));
                print_colored("  purpose: ", DIM);
                println!("{}", ctx.purpose);
                if let Some(preview) = ctx.prompt_preview {
                    print_colored("  prompt: ", DIM);
                    println!("{}...", preview.chars().take(80).collect::<String>());
                }
            } else {
                println!("{}", step.content);
            }
            println!();
        }
    }
}

/// Print a single dialogue step (streaming mode)
pub fn print_step(step: &anna_shared::rpc::DialogueStep) {
    print_step_internal(step, false);
}

/// Print the full dialogue for transparency
#[allow(dead_code)]
pub fn print_dialogue(result: &AskResult) {
    for step in &result.dialogue {
        print_step_internal(step, true);
    }
}

/// Print timeout error box
pub fn print_timeout_error(timeout_secs: u64) {
    println!();
    println_colored("╔══════════════════════════════════════════════╗", RED);
    println_colored("║           REQUEST TIMED OUT                  ║", RED);
    println_colored("╚══════════════════════════════════════════════╝", RED);
    println!();
    println!("  The request took longer than {}s to complete.", timeout_secs);
    println!();
    println_colored("  Possible causes:", YELLOW);
    println!("  • Ollama model is loading (first query is slow)");
    println!("  • Complex question requiring many iterations");
    println!("  • LLM server is overloaded");
    println!();
    println_colored("  Suggestions:", GREEN);
    println!("  • Try again - model may be loaded now");
    println!("  • Check: annactl status");
    println!("  • Increase timeout in ~/.anna/config.toml:");
    println!("    [performance]");
    println!("    llm_timeout_secs = 180");
    println!();
}

/// Flush stdout
pub fn flush_stdout() {
    io::stdout().flush().ok();
}

/// Print a box with content
fn print_box(content: &str) {
    println_colored("┌─────────────────────────────────────────", DIM);
    for line in content.lines() {
        println_colored(&format!("│ {}", line), DIM);
    }
    println_colored("└─────────────────────────────────────────", DIM);
}

/// Print command list
fn print_commands(content: &str) {
    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty() {
            print_colored("  $ ", DIM);
            println_colored(line, CYAN);
        }
    }
}
