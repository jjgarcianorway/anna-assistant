//! Display utilities for CLI output - colors, step printing, status display.
//! v0.0.992: Added proactive alert display

use anna_shared::monitor::{IssueStore, Severity};
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

/// v0.0.992: Show proactive alerts from monitoring system
/// Returns true if any alerts were shown
pub fn show_proactive_alerts() -> bool {
    let store = match IssueStore::load() {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Get unacknowledged issues
    let critical: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Critical && !i.acknowledged)
        .collect();
    let warnings: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Warning && !i.acknowledged)
        .collect();

    if critical.is_empty() && warnings.is_empty() {
        return false;
    }

    println!();

    // Show critical issues in natural language
    if !critical.is_empty() {
        print_colored("I noticed some issues while monitoring your system:\n", YELLOW);
        println!();
        for issue in &critical {
            print_colored("  ✗ ", RED);
            println!("{}", issue.summary);
            if let Some(ref fix) = issue.suggested_fix {
                print_colored("    → ", DIM);
                println_colored(fix, DIM);
            }
        }
    }

    // Show warnings
    if !warnings.is_empty() {
        if critical.is_empty() {
            print_colored("Heads up - I noticed a few things:\n", YELLOW);
            println!();
        }
        for issue in warnings.iter().take(3) { // Limit to 3 warnings
            print_colored("  ⚠ ", YELLOW);
            println!("{}", issue.summary);
        }
        if warnings.len() > 3 {
            println_colored(&format!("  ... and {} more", warnings.len() - 3), DIM);
        }
    }

    println!();
    true
}

/// v0.0.992: Mark alerts as notified after showing them
pub fn mark_alerts_shown() {
    if let Ok(mut store) = IssueStore::load() {
        store.mark_notified();
        let _ = store.save();
    }
}

/// v0.0.998: Print stats about Anna's activity
pub fn print_stats() {
    println!();
    println_colored("Anna Statistics", BOLD);
    println_colored("═══════════════════════════════════════", DIM);
    println!();

    // 1. Fix history
    let fix_history_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("anna/fix_history.json");

    if fix_history_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&fix_history_path) {
            if let Ok(history) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(fixes) = history.get("fixes").and_then(|f| f.as_array()) {
                    print_colored("Automatic Fixes: ", CYAN);
                    println_colored(&format!("{}", fixes.len()), GREEN);

                    // Show last few fixes
                    for fix in fixes.iter().rev().take(3) {
                        if let Some(id) = fix.get("fix_id").and_then(|v| v.as_str()) {
                            if let Some(ts) = fix.get("timestamp").and_then(|v| v.as_str()) {
                                let short_ts = ts.split('T').next().unwrap_or(ts);
                                print_colored("  • ", DIM);
                                print!("{}", id);
                                print_colored(&format!(" ({})", short_ts), DIM);
                                println!();
                            }
                        }
                    }
                }
            }
        }
    } else {
        print_colored("Automatic Fixes: ", CYAN);
        println_colored("0", DIM);
    }

    println!();

    // 2. Change history (recipes applied)
    let changes_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("anna/changes.json");

    if changes_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&changes_path) {
            if let Ok(history) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(changes) = history.get("changes").and_then(|c| c.as_array()) {
                    let undoable: Vec<_> = changes.iter()
                        .filter(|c| c.get("undone").and_then(|u| u.as_bool()).unwrap_or(false) == false)
                        .collect();

                    print_colored("Configuration Changes: ", CYAN);
                    println_colored(&format!("{}", changes.len()), GREEN);
                    print_colored("  Undoable: ", DIM);
                    println!("{}", undoable.len());

                    // Show last few changes
                    for change in changes.iter().rev().take(3) {
                        if let Some(name) = change.get("name").and_then(|v| v.as_str()) {
                            if let Some(cat) = change.get("category").and_then(|v| v.as_str()) {
                                print_colored("  • ", DIM);
                                print!("{}", name);
                                print_colored(&format!(" [{}]", cat), DIM);
                                if change.get("undone").and_then(|u| u.as_bool()).unwrap_or(false) {
                                    print_colored(" (undone)", YELLOW);
                                }
                                println!();
                            }
                        }
                    }
                }
            }
        }
    } else {
        print_colored("Configuration Changes: ", CYAN);
        println_colored("0", DIM);
    }

    println!();

    // 3. Memory experiences
    let memory_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("anna/memory.json");

    if memory_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&memory_path) {
            if let Ok(memory) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(exp) = memory.get("experiences").and_then(|e| e.as_array()) {
                    print_colored("Learned Experiences: ", CYAN);
                    println_colored(&format!("{}", exp.len()), GREEN);
                }
            }
        }
    } else {
        print_colored("Learned Experiences: ", CYAN);
        println_colored("0", DIM);
    }

    println!();

    // 4. Installed dependencies (tools Anna installed)
    let deps_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".anna/installed_deps.txt");

    if deps_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&deps_path) {
            let deps: Vec<_> = content.lines().filter(|l| !l.is_empty()).collect();
            if !deps.is_empty() {
                print_colored("Installed Tools: ", CYAN);
                println_colored(&format!("{}", deps.len()), GREEN);
                for dep in deps.iter().take(5) {
                    print_colored("  • ", DIM);
                    println!("{}", dep);
                }
                if deps.len() > 5 {
                    println_colored(&format!("  ... and {} more", deps.len() - 5), DIM);
                }
            }
        }
    } else {
        print_colored("Installed Tools: ", CYAN);
        println_colored("0", DIM);
    }

    println!();
    println_colored("═══════════════════════════════════════", DIM);
    println!();
}
