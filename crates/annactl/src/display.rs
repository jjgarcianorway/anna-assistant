//! Display utilities for CLI output - colors, step printing, status display.
//! v0.0.992: Added proactive alert display
//! v0.1.0: Added debug mode separation for clean "fly on the wall" experience

use anna_shared::config::AnnaConfig;
use anna_shared::monitor::{IssueStore, Severity};
use anna_shared::rpc::{AskResult, StepType};
use std::io::{self, Write};
use std::sync::OnceLock;

/// Cached debug mode flag (loaded once at startup)
static DEBUG_MODE: OnceLock<bool> = OnceLock::new();

/// Check if debug mode is enabled (cached)
fn is_debug_mode() -> bool {
    *DEBUG_MODE.get_or_init(|| {
        AnnaConfig::load().map(|c| c.debug_mode).unwrap_or(true)
    })
}

// Color constants
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[37;1m"; // v0.0.999: Bright white for tickets
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

/// Print status - v0.1.0: Comprehensive IT Department status display
pub async fn print_status() {
    match crate::rpc::get_status().await {
        Ok(status) => {
            let config = AnnaConfig::load().ok();
            let debug_mode = config.as_ref().map(|c| c.debug_mode).unwrap_or(false);

            println!();
            println_colored("╔═══════════════════════════════════════════════════════════╗", CYAN);
            println_colored("║           ANNA - IT DEPARTMENT STATUS                     ║", CYAN);
            println_colored("╚═══════════════════════════════════════════════════════════╝", CYAN);
            println!();

            // ════════════════════════════════════════════════════════════
            // VERSION & UPDATE INFO
            // ════════════════════════════════════════════════════════════
            println_colored("┌─ VERSION ────────────────────────────────────────────────┐", DIM);
            print_colored("  Installed:        ", DIM);
            println_colored(&status.version, GREEN);

            // Latest version from GitHub
            if let Some(ref latest) = status.latest_version {
                print_colored("  Available:        ", DIM);
                if latest != &status.version {
                    print_colored(latest, YELLOW);
                    println_colored(" (update available)", YELLOW);
                } else {
                    print_colored(latest, GREEN);
                    println_colored(" (up to date)", DIM);
                }
            }

            // Update schedule
            print_colored("  Update Check:     ", DIM);
            println!("Every {} seconds", status.update_check_interval_secs);

            // Last/next check
            if let Some(ref last) = status.last_update_check {
                print_colored("  Last Check:       ", DIM);
                println_colored(&format_time_ago(last), DIM);
            }

            println_colored("└───────────────────────────────────────────────────────────┘", DIM);
            println!();

            // ════════════════════════════════════════════════════════════
            // DAEMON & ENVIRONMENT
            // ════════════════════════════════════════════════════════════
            println_colored("┌─ ENVIRONMENT ────────────────────────────────────────────┐", DIM);

            // Daemon state
            print_colored("  Daemon:           ", DIM);
            let state_color = match status.state {
                anna_shared::status::DaemonState::Ready => GREEN,
                anna_shared::status::DaemonState::Starting => YELLOW,
                anna_shared::status::DaemonState::Error => RED,
            };
            print_colored("● ", state_color);
            print_colored(&status.state.to_string().to_lowercase(), state_color);
            println_colored(&format!(" (uptime: {})", format_duration(status.uptime_secs)), DIM);

            // Ollama
            print_colored("  Ollama:           ", DIM);
            if status.ollama_running {
                print_colored("● ", GREEN);
                print_colored("running", GREEN);
                if let Some(model) = &status.model {
                    println_colored(&format!(" ({})", model), DIM);
                } else {
                    println!();
                }
            } else {
                print_colored("○ ", RED);
                println_colored("not running", RED);
            }

            // GPU
            if let Some(gpu) = &status.gpu {
                print_colored("  GPU:              ", DIM);
                print_colored(gpu, CYAN);
                if let Some(vram) = status.vram_mb {
                    println_colored(&format!(" ({} MB VRAM)", vram), DIM);
                } else {
                    println!();
                }
            }

            // User groups (gathered locally)
            print_colored("  User Groups:      ", DIM);
            println_colored(&get_user_groups(), DIM);

            println_colored("└───────────────────────────────────────────────────────────┘", DIM);
            println!();

            // ════════════════════════════════════════════════════════════
            // KNOWLEDGE & LEARNING
            // ════════════════════════════════════════════════════════════
            println_colored("┌─ KNOWLEDGE ──────────────────────────────────────────────┐", DIM);

            // Patterns
            print_colored("  Patterns:         ", DIM);
            print_colored(&format!("{}", status.pattern_count), GREEN);
            println_colored(" built-in", DIM);

            // Recipes
            print_colored("  Recipes:          ", DIM);
            print_colored(&format!("{}", status.recipe_count), GREEN);
            println_colored(" learned", DIM);

            // Memory
            print_colored("  Memory:           ", DIM);
            if status.memory_experiences == 0 {
                println_colored("empty (learning from interactions)", DIM);
            } else {
                print_colored(&format!("{}", status.memory_experiences), GREEN);
                println_colored(" experiences", DIM);
            }

            // Show health issues if any
            for issue in &status.memory_health_issues {
                print_colored("    ⚠ ", YELLOW);
                println_colored(issue, YELLOW);
            }

            println_colored("└───────────────────────────────────────────────────────────┘", DIM);
            println!();

            // ════════════════════════════════════════════════════════════
            // HELPERS (tools Anna has installed)
            // ════════════════════════════════════════════════════════════
            let helpers = get_helpers_list();
            if !helpers.is_empty() {
                println_colored("┌─ HELPERS ─────────────────────────────────────────────────┐", DIM);
                for (name, by_anna) in &helpers {
                    print_colored("  ", DIM);
                    print_colored("● ", GREEN);
                    print_colored(&format!("{:16}", name), DIM);
                    if *by_anna {
                        println_colored("[anna]", CYAN);
                    } else {
                        println_colored("[user]", DIM);
                    }
                }
                println_colored("└───────────────────────────────────────────────────────────┘", DIM);
                println!();
            }

            // ════════════════════════════════════════════════════════════
            // CONFIGURATION
            // ════════════════════════════════════════════════════════════
            println_colored("┌─ CONFIG ───────────────────────────────────────────────────┐", DIM);

            print_colored("  Debug Mode:       ", DIM);
            if debug_mode {
                println_colored("ON (showing internal details)", YELLOW);
            } else {
                println_colored("OFF (clean dialogue mode)", GREEN);
            }

            if let Some(ref cfg) = config {
                print_colored("  Auto Helpers:     ", DIM);
                println_colored(if cfg.auto_install_helpers { "ON" } else { "OFF" }, if cfg.auto_install_helpers { GREEN } else { DIM });

                print_colored("  Ask Clarification:", DIM);
                println_colored(if cfg.ask_clarification { "ON" } else { "OFF" }, if cfg.ask_clarification { GREEN } else { DIM });

                print_colored("  Wiki Embeddings:  ", DIM);
                println_colored(if cfg.wiki.use_embeddings { "ON" } else { "OFF" }, if cfg.wiki.use_embeddings { GREEN } else { DIM });
            }

            println_colored("└───────────────────────────────────────────────────────────┘", DIM);
            println!();

            // Tip
            println_colored("Tip: 'annactl stats' for RPG progression & detailed statistics", DIM);
            println!();
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
}

/// Format seconds as human-readable duration
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Format RFC3339 timestamp as "X ago"
fn format_time_ago(rfc3339: &str) -> String {
    use chrono::{DateTime, Utc};
    if let Ok(dt) = DateTime::parse_from_rfc3339(rfc3339) {
        let now = Utc::now();
        let diff = now.signed_duration_since(dt.with_timezone(&Utc));
        let secs = diff.num_seconds();
        if secs < 0 {
            return "just now".to_string();
        }
        format_duration(secs as u64) + " ago"
    } else {
        rfc3339.to_string()
    }
}

/// Get user groups
fn get_user_groups() -> String {
    std::process::Command::new("groups")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get list of helpers and whether they were installed by Anna
fn get_helpers_list() -> Vec<(String, bool)> {
    let deps_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".anna/installed_deps.txt");

    let anna_installed: std::collections::HashSet<String> = if deps_path.exists() {
        std::fs::read_to_string(&deps_path)
            .ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    // Check for common diagnostic tools Anna might use
    let tools = ["nethogs", "iotop", "htop", "lsof", "strace", "bc", "jq", "yq", "fzf"];
    let mut result = Vec::new();

    for tool in tools {
        // Check if tool exists in PATH
        if std::process::Command::new("which")
            .arg(tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let by_anna = anna_installed.contains(tool);
            result.push((tool.to_string(), by_anna));
        }
    }

    result
}

/// Internal step printer with option to force FinalAnswer content
/// v0.1.0: Respects debug_mode - when false, only shows "fly on the wall" narrative
fn print_step_internal(step: &anna_shared::rpc::DialogueStep, force_final_answer: bool) {
    let debug = is_debug_mode();

    match step.step_type {
        // ═══════════════════════════════════════════════════════════════════
        // ALWAYS VISIBLE (both debug and normal mode)
        // These are the "fly on the wall" experience elements
        // ═══════════════════════════════════════════════════════════════════
        StepType::UserQuestion => {
            print_colored("USER: ", CYAN);
            println!("{}", step.content);
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
            // Errors always shown, but with less detail in normal mode
            if debug {
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
            } else {
                // Simplified error in normal mode
                print_colored("  ✗ ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println_colored(&ctx.message, RED);
                } else {
                    println_colored("An error occurred while processing", RED);
                }
            }
            println!();
        }
        // v0.0.999: IT Department team dialogue - ALWAYS VISIBLE (the Hollywood experience!)
        StepType::TicketCreated => {
            println!();
            print_colored("┌─ ", DIM);
            print_colored("TICKET ", CYAN);
            println_colored(&step.content, WHITE);
            print_colored("└─", DIM);
            println!();
        }
        StepType::TeamAssignment => {
            print_colored("Anna → ", MAGENTA);
            println!("{}", step.content);
        }
        StepType::TeamDialogue => {
            print_colored("  │ ", DIM);
            println!("{}", step.content);
        }
        StepType::TeamEscalation => {
            println!();
            print_colored("  ⇈ ESCALATING: ", YELLOW);
            println!("{}", step.content);
        }

        // ═══════════════════════════════════════════════════════════════════
        // DEBUG ONLY - Hidden in normal mode for clean "fly on the wall" UX
        // These show the internal workings (raw commands, prompts, etc.)
        // ═══════════════════════════════════════════════════════════════════
        StepType::AnnaToLlm => {
            if debug {
                print_colored("ANNA → LLM: ", YELLOW);
                println!("(command selection prompt)");
                print_box(&step.content);
                println!();
            }
        }
        StepType::LlmCommands => {
            if debug {
                print_colored("LLM → ANNA: ", YELLOW);
                if step.content == "NONE" || step.content == "DONE" {
                    println_colored(&step.content, DIM);
                } else {
                    println!("commands to run:");
                    print_commands(&step.content);
                }
                println!();
            }
        }
        StepType::CommandExec => {
            if debug {
                print_colored("EXEC: ", GREEN);
                println!("{}", step.content);
            }
        }
        StepType::CommandOutput => {
            if debug {
                print_colored("OUTPUT: ", DIM);
                println!("{}", step.content);
                println!();
            }
        }
        StepType::ValidationPrompt => {
            if debug {
                print_colored("ANNA → LLM: ", YELLOW);
                println!("(validation prompt)");
                print_box(&step.content);
                println!();
            }
        }
        StepType::ValidationResponse => {
            if debug {
                print_colored("LLM → ANNA: ", YELLOW);
                println!("{}", step.content);
                println!();
            }
        }
        StepType::FinalPrompt => {
            if debug {
                print_colored("ANNA → LLM: ", YELLOW);
                println!("(final answer prompt)");
                print_box(&step.content);
                println!();
            }
        }
        StepType::WikiSearch => {
            // Show brief wiki search in normal mode, full in debug
            if debug {
                print_colored("ANNA → WIKI: ", MAGENTA);
                println!("searching Arch Wiki...");
                println_colored(&format!("  query: {}", step.content), DIM);
                println!();
            } else {
                // Just a brief note that we're checking the wiki
                print_colored("  │ ", DIM);
                println_colored("Checking Arch Wiki...", DIM);
            }
        }
        StepType::WikiResults => {
            if debug {
                print_colored("WIKI → ANNA: ", MAGENTA);
                println!("found articles:");
                for line in step.content.lines() {
                    println_colored(&format!("  • {}", line), DIM);
                }
                println!();
            }
        }
        StepType::WikiCommands => {
            if debug {
                print_colored("WIKI: ", MAGENTA);
                println!("extracted commands:");
                print_commands(&step.content);
                println!();
            }
        }
        StepType::IntentResult => {
            // Show intent in both modes but differently
            if debug {
                print_colored("  intent: ", DIM);
                println!("{}", step.content);
            } else {
                // In normal mode, just show the key info
                print_colored("  ", DIM);
                println_colored(&step.content, DIM);
            }
        }
        StepType::SubQuestion => {
            if debug {
                println!();
                print_colored("─── ", DIM);
                print_colored(&step.content, YELLOW);
                println!();
            }
        }
        StepType::SubQuestionResult => {
            if debug {
                print_colored("  → ", GREEN);
                println!("{}", step.content);
            }
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

/// v0.0.999: Print comprehensive stats about Anna's activity
/// Includes RPG progression, department stats, and performance metrics
pub fn print_stats() {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("anna");

    println!();
    println_colored("╔═══════════════════════════════════════════════════════════╗", CYAN);
    println_colored("║           ANNA - IT DEPARTMENT STATISTICS                 ║", CYAN);
    println_colored("╚═══════════════════════════════════════════════════════════╝", CYAN);
    println!();

    // ═══════════════════════════════════════════════════════════
    // RPG PROGRESSION
    // ═══════════════════════════════════════════════════════════
    let xp_path = data_dir.join("xp.json");
    let (level, total_xp, title, title_desc, progress, tickets_resolved, resolved_by_anna, recipes_learned) =
        if xp_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&xp_path) {
                if let Ok(xp) = serde_json::from_str::<serde_json::Value>(&content) {
                    let level = xp.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                    let total = xp.get("total_xp").and_then(|v| v.as_u64()).unwrap_or(0);
                    let tickets = xp.get("tickets_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                    let by_anna = xp.get("resolved_by_anna").and_then(|v| v.as_u64()).unwrap_or(0);
                    let recipes = xp.get("recipes_learned").and_then(|v| v.as_u64()).unwrap_or(0);
                    let title = get_title_for_level(level);
                    let desc = get_title_description(level);
                    // Calculate progress percentage
                    let xp_needed = xp_for_level(level + 1);
                    let xp_current = xp_for_level(level);
                    let prog = if xp_needed > xp_current {
                        ((total.saturating_sub(xp_current)) as f64 / (xp_needed - xp_current) as f64 * 100.0).min(100.0)
                    } else { 100.0 };
                    (level, total, title, desc, prog, tickets, by_anna, recipes)
                } else {
                    (1, 0, "Helpdesk Newbie", "Just starting out!", 0.0, 0, 0, 0)
                }
            } else {
                (1, 0, "Helpdesk Newbie", "Just starting out!", 0.0, 0, 0, 0)
            }
        } else {
            (1, 0, "Helpdesk Newbie", "Just starting out!", 0.0, 0, 0, 0)
        };

    println_colored("┌─ ANNA'S PROGRESSION ─────────────────────────────────────┐", DIM);
    println!();
    print_colored("  Level: ", CYAN);
    print_colored(&format!("{}", level), BOLD);
    print_colored(" / 100", DIM);
    print!("  ");
    print_colored(&format!("\"{}\"", title), YELLOW);
    println!();
    println_colored(&format!("  {}", title_desc), DIM);
    println!();

    // XP progress bar
    print_colored("  XP: ", CYAN);
    print!("{} ", total_xp);
    print_colored("[", DIM);
    let bar_width = 30;
    let filled = (progress / 100.0 * bar_width as f64) as usize;
    print_colored(&"█".repeat(filled), GREEN);
    print_colored(&"░".repeat(bar_width - filled), DIM);
    print_colored("]", DIM);
    println_colored(&format!(" {:.0}%", progress), GREEN);
    println!();

    // Quick stats
    print_colored("  Tickets Resolved: ", DIM);
    print_colored(&format!("{}", tickets_resolved), GREEN);
    print!("  ");
    print_colored("By Anna: ", DIM);
    print_colored(&format!("{}", resolved_by_anna), GREEN);
    print!("  ");
    print_colored("Recipes Learned: ", DIM);
    println_colored(&format!("{}", recipes_learned), GREEN);

    println!();
    println_colored("└───────────────────────────────────────────────────────────┘", DIM);
    println!();

    // ═══════════════════════════════════════════════════════════
    // IT DEPARTMENT TEAM
    // ═══════════════════════════════════════════════════════════
    println_colored("┌─ IT DEPARTMENT TEAM ─────────────────────────────────────┐", DIM);
    println!();
    print_team_member("Network", "Michael", "Junior", "Sarah", "Senior");
    print_team_member("Desktop", "Alex", "Junior", "Emma", "Senior");
    print_team_member("System", "James", "Junior", "Lisa", "Senior");
    print_team_member("Packages", "David", "Junior", "Nina", "Senior");
    print_team_member("Hardware", "Ryan", "Junior", "Sophie", "Senior");
    print_team_member("Audio", "Chris", "Junior", "Maria", "Senior");
    print_team_member("Storage", "Kevin", "Junior", "Rachel", "Senior");
    print_team_member("Security", "Tom", "Junior", "Elena", "Senior");
    println!();
    print_colored("  Team Size: ", DIM);
    println_colored("16 specialists (8 departments)", GREEN);
    println!();
    println_colored("└───────────────────────────────────────────────────────────┘", DIM);
    println!();

    // ═══════════════════════════════════════════════════════════
    // ACTIVITY STATISTICS
    // ═══════════════════════════════════════════════════════════
    println_colored("┌─ ACTIVITY ───────────────────────────────────────────────┐", DIM);
    println!();

    // Fix history
    let fix_history_path = data_dir.join("fix_history.json");
    let fixes_count = if fix_history_path.exists() {
        std::fs::read_to_string(&fix_history_path).ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|h| h.get("fixes").and_then(|f| f.as_array()).map(|a| a.len()))
            .unwrap_or(0)
    } else { 0 };

    print_colored("  Automatic Fixes Applied: ", DIM);
    println_colored(&format!("{}", fixes_count), GREEN);

    // Changes
    let changes_path = data_dir.join("changes.json");
    let (changes_count, undoable) = if changes_path.exists() {
        std::fs::read_to_string(&changes_path).ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|h| {
                h.get("changes").and_then(|c| c.as_array()).map(|arr| {
                    let total = arr.len();
                    let undo = arr.iter()
                        .filter(|c| !c.get("undone").and_then(|u| u.as_bool()).unwrap_or(false))
                        .count();
                    (total, undo)
                })
            })
            .unwrap_or((0, 0))
    } else { (0, 0) };

    print_colored("  Configuration Changes: ", DIM);
    print_colored(&format!("{}", changes_count), GREEN);
    print_colored(&format!(" ({} undoable)", undoable), DIM);
    println!();

    // Memory
    let memory_path = data_dir.join("memory.json");
    let exp_count = if memory_path.exists() {
        std::fs::read_to_string(&memory_path).ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|m| m.get("experiences").and_then(|e| e.as_array()).map(|a| a.len()))
            .unwrap_or(0)
    } else { 0 };

    print_colored("  Learned Experiences: ", DIM);
    println_colored(&format!("{}", exp_count), GREEN);

    // Helpers
    let deps_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".anna/installed_deps.txt");
    let helpers_count = if deps_path.exists() {
        std::fs::read_to_string(&deps_path).ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).count())
            .unwrap_or(0)
    } else { 0 };

    print_colored("  Installed Helpers: ", DIM);
    print_colored(&format!("{}", helpers_count), GREEN);
    println_colored(" (by Anna)", DIM);

    println!();
    println_colored("└───────────────────────────────────────────────────────────┘", DIM);
    println!();

    // ═══════════════════════════════════════════════════════════
    // TICKET STATS (if available)
    // ═══════════════════════════════════════════════════════════
    let tickets_path = data_dir.join("tickets.json");
    if tickets_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&tickets_path) {
            if let Ok(store) = serde_json::from_str::<serde_json::Value>(&content) {
                println_colored("┌─ TICKET METRICS ──────────────────────────────────────────┐", DIM);
                println!();

                let total_resolved = store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                let total_failed = store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0);
                let total_escalated = store.get("total_escalated").and_then(|v| v.as_u64()).unwrap_or(0);

                print_colored("  Total Resolved: ", DIM);
                print_colored(&format!("{}", total_resolved), GREEN);
                print!("  ");
                print_colored("Failed: ", DIM);
                print_colored(&format!("{}", total_failed), if total_failed > 0 { RED } else { DIM });
                print!("  ");
                print_colored("Escalated: ", DIM);
                println_colored(&format!("{}", total_escalated), YELLOW);

                if total_resolved > 0 {
                    let success_rate = total_resolved as f64 / (total_resolved + total_failed) as f64 * 100.0;
                    print_colored("  Success Rate: ", DIM);
                    let rate_color = if success_rate >= 90.0 { GREEN } else if success_rate >= 70.0 { YELLOW } else { RED };
                    println_colored(&format!("{:.1}%", success_rate), rate_color);
                }

                println!();
                println_colored("└───────────────────────────────────────────────────────────┘", DIM);
                println!();
            }
        }
    }

    println_colored("Tip: Use 'annactl status' for daemon info, 'annactl stats' for this view", DIM);
    println!();
}

/// Helper to print team member row
fn print_team_member(dept: &str, jr_name: &str, jr_role: &str, sr_name: &str, sr_role: &str) {
    print_colored(&format!("  {:10}", dept), CYAN);
    print_colored(&format!("{:8}", jr_name), DIM);
    print_colored(&format!("({})", jr_role), DIM);
    print!("  ");
    print_colored(&format!("{:8}", sr_name), DIM);
    println_colored(&format!("({})", sr_role), DIM);
}

/// Get title for level (RPG style)
fn get_title_for_level(level: u32) -> &'static str {
    match level {
        0..=5 => "Helpdesk Newbie",
        6..=10 => "Support Rookie",
        11..=15 => "Tech Apprentice",
        16..=20 => "Junior Analyst",
        21..=25 => "IT Assistant",
        26..=30 => "System Helper",
        31..=35 => "Tech Support Pro",
        36..=40 => "Senior Analyst",
        41..=45 => "IT Specialist",
        46..=50 => "System Expert",
        51..=55 => "Tech Guru",
        56..=60 => "IT Veteran",
        61..=65 => "System Master",
        66..=70 => "Tech Wizard",
        71..=75 => "IT Sage",
        76..=80 => "System Oracle",
        81..=85 => "Tech Legend",
        86..=90 => "IT Deity",
        91..=95 => "System Overlord",
        96..=99 => "Tech Transcendent",
        100 => "The One Who Knows All",
        _ => "Unknown Entity",
    }
}

/// Get description for level
fn get_title_description(level: u32) -> &'static str {
    match level {
        0..=5 => "Just starting out, but eager to help!",
        6..=10 => "Learning the ropes, one ticket at a time.",
        11..=15 => "Getting the hang of this IT thing.",
        16..=20 => "Can handle most basic requests now.",
        21..=25 => "The go-to for everyday tech problems.",
        26..=30 => "Knows the system like the back of the hand.",
        31..=35 => "Rarely needs to escalate anymore.",
        36..=40 => "The specialists come to me for advice!",
        41..=45 => "One with the terminal, one with the code.",
        46..=50 => "Halfway to omniscience.",
        51..=55 => "The Arch Wiki fears me.",
        56..=60 => "Bugs flee at my presence.",
        61..=65 => "I dream in systemd unit files.",
        66..=70 => "The kernel sends me birthday cards.",
        71..=75 => "Linus would be proud.",
        76..=80 => "I see the Matrix now.",
        81..=85 => "Compiling wisdom since boot.",
        86..=90 => "The Arch Wiki quotes ME.",
        91..=95 => "I AM the documentation.",
        96..=99 => "One step from digital enlightenment.",
        100 => "I have achieved technical nirvana.",
        _ => "An enigma wrapped in a shell script.",
    }
}

/// Calculate XP needed for level
fn xp_for_level(level: u32) -> u64 {
    let base = 100.0;
    let xp = base * (level as f64).powf(1.5) + (level as f64 * 50.0);
    xp as u64
}
