//! Display utilities for CLI output - colors, step printing, status display.
//! v0.0.992: Added proactive alert display
//! v0.1.0: Added debug mode separation for clean "fly on the wall" experience
//! v0.1.1: Removed box drawing for cleaner look

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
pub const WHITE: &str = "\x1b[37;1m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

/// Print colored text (no newline)
pub fn print_colored(text: &str, color: &str) {
    print!("{}{}{}", color, text, RESET);
}

/// Print colored text with newline
pub fn println_colored(text: &str, color: &str) {
    println!("{}{}{}", color, text, RESET);
}

/// Print the greeting
pub fn print_greeting() {
    println!();
    println_colored("Anna - Arch Linux Assistant", BOLD);
    println_colored("Ask questions about your system in plain English.", DIM);
    println_colored("Type 'quit' or Ctrl-D to exit.", DIM);
    println!();
}

/// Print status - clean format without boxes
pub async fn print_status() {
    match crate::rpc::get_status().await {
        Ok(status) => {
            let config = AnnaConfig::load().ok();
            let debug_mode = config.as_ref().map(|c| c.debug_mode).unwrap_or(false);

            println!();
            println_colored("ANNA STATUS", BOLD);
            println!();

            // VERSION
            println_colored("VERSION", CYAN);
            print!("  Installed:     ");
            println_colored(&status.version, GREEN);

            if let Some(ref latest) = status.latest_version {
                print!("  Available:     ");
                if latest != &status.version {
                    print_colored(latest, YELLOW);
                    println_colored(" (update available)", YELLOW);
                } else {
                    print_colored(latest, GREEN);
                    println_colored(" ✓", GREEN);
                }
            }
            println!();

            // ENVIRONMENT
            println_colored("ENVIRONMENT", CYAN);

            print!("  Daemon:        ");
            let state_color = match status.state {
                anna_shared::status::DaemonState::Ready => GREEN,
                anna_shared::status::DaemonState::Starting => YELLOW,
                anna_shared::status::DaemonState::Error => RED,
            };
            print_colored(&status.state.to_string().to_lowercase(), state_color);
            println_colored(&format!(" (uptime: {})", format_duration(status.uptime_secs)), DIM);

            print!("  Ollama:        ");
            if status.ollama_running {
                print_colored("running", GREEN);
                if let Some(model) = &status.model {
                    println_colored(&format!(" ({})", model), DIM);
                } else {
                    println!();
                }
            } else {
                println_colored("not running", RED);
            }

            if let Some(gpu) = &status.gpu {
                print!("  GPU:           ");
                print_colored(gpu, CYAN);
                if let Some(vram) = status.vram_mb {
                    println_colored(&format!(" ({} MB)", vram), DIM);
                } else {
                    println!();
                }
            }
            println!();

            // KNOWLEDGE
            println_colored("KNOWLEDGE", CYAN);
            println!("  Patterns:      {} built-in", status.pattern_count);
            println!("  Recipes:       {} learned", status.recipe_count);
            print!("  Memory:        ");
            if status.memory_experiences == 0 {
                println_colored("empty", DIM);
            } else {
                println!("{} experiences", status.memory_experiences);
            }

            for issue in &status.memory_health_issues {
                print_colored("    ⚠ ", YELLOW);
                println_colored(issue, YELLOW);
            }
            println!();

            // HELPERS
            let helpers = get_helpers_list();
            if !helpers.is_empty() {
                println_colored("HELPERS", CYAN);
                for (name, by_anna) in &helpers {
                    print!("  ");
                    print_colored(&format!("{:16}", name), DIM);
                    if *by_anna {
                        println_colored("[anna]", CYAN);
                    } else {
                        println_colored("[user]", DIM);
                    }
                }
                println!();
            }

            // RPG STATS
            println_colored("STATS", CYAN);
            let rpg = &status.rpg_stats;

            // Title and XP bar
            print!("  ");
            print_colored(&rpg.title, MAGENTA);
            print!(" ");
            println_colored(&rpg.xp_bar(), DIM);

            // Questions answered
            if rpg.total_questions > 0 {
                print!("  Questions:     ");
                print!("{}", rpg.total_questions);
                if rpg.instant_answers > 0 || rpg.memory_answers > 0 {
                    let fast = rpg.instant_answers + rpg.memory_answers;
                    let pct = (fast as f64 / rpg.total_questions as f64 * 100.0) as u32;
                    print_colored(&format!(" ({}% instant)", pct), DIM);
                }
                println!();
            }

            // Response times
            if rpg.avg_response_ms > 0 {
                print!("  Response:      ");
                print!("avg {}ms", rpg.avg_response_ms);
                println_colored(&format!(" (fast: {}ms, slow: {}ms)", rpg.fastest_response_ms, rpg.slowest_response_ms), DIM);
            }

            // Recipes learned
            if rpg.recipes_learned > 0 {
                println!("  Recipes:       {} learned", rpg.recipes_learned);
            }

            // Reliability
            if rpg.total_questions > 10 {
                print!("  Reliability:   ");
                let rel_pct = (rpg.reliability * 100.0) as u32;
                let rel_color = if rel_pct >= 90 { GREEN } else if rel_pct >= 70 { YELLOW } else { RED };
                println_colored(&format!("{}%", rel_pct), rel_color);
            }

            // Total uptime
            if rpg.total_uptime_secs > 0 {
                print!("  Total uptime:  ");
                println_colored(&format_duration(rpg.total_uptime_secs), DIM);
            }
            println!();

            // CONFIG
            println_colored("CONFIG", CYAN);
            print!("  Debug Mode:    ");
            if debug_mode {
                println_colored("ON", YELLOW);
            } else {
                println_colored("OFF", GREEN);
            }

            if let Some(ref cfg) = config {
                print!("  Auto Helpers:  ");
                println_colored(if cfg.auto_install_helpers { "ON" } else { "OFF" },
                    if cfg.auto_install_helpers { GREEN } else { DIM });
            }
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

    let tools = ["nethogs", "iotop", "htop", "lsof", "strace", "bc", "jq", "yq", "fzf"];
    let mut result = Vec::new();

    for tool in tools {
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

/// Print a single dialogue step
fn print_step_internal(step: &anna_shared::rpc::DialogueStep, force_final_answer: bool) {
    let debug = is_debug_mode();

    match step.step_type {
        // ALWAYS VISIBLE
        StepType::UserQuestion => {
            print_colored("You: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::FinalAnswer => {
            if !step.content.is_empty() || force_final_answer {
                println!();
                print_colored("Anna: ", GREEN);
                if force_final_answer {
                    println!();
                }
                println!("{}", step.content);
                println!();
            }
        }
        StepType::ClarificationQuestion => {
            print_colored("Anna: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::ClarificationResponse => {
            print_colored("You: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::IntentClassifying => {
            if debug {
                println_colored("  understanding question...", DIM);
            }
        }
        StepType::UnderstandingCheck => {
            print_colored("Anna: ", CYAN);
            println!("{}", step.content);
        }
        StepType::ConfirmationRequest => {
            println!();
            print_colored("Anna: ", YELLOW);
            println!("Please confirm:");
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::MissingInfo => {
            print_colored("Anna: ", RED);
            println!("Missing information:");
            for line in step.content.lines() {
                println!("  - {}", line);
            }
        }
        StepType::SystemAlert => {
            println!();
            println_colored("SYSTEM ALERT", YELLOW);
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::LlmError => {
            if debug {
                print_colored("Error: ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println!("{}", ctx.message);
                } else {
                    println!("{}", step.content);
                }
            } else {
                print_colored("  ✗ ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println_colored(&ctx.message, RED);
                } else {
                    println_colored("An error occurred", RED);
                }
            }
            println!();
        }
        // Team dialogue (always visible - fly on the wall)
        StepType::TicketCreated => {
            println!();
            print_colored("Ticket ", CYAN);
            println_colored(&step.content, WHITE);
        }
        StepType::TeamAssignment => {
            print_colored("Anna → ", MAGENTA);
            println!("{}", step.content);
        }
        StepType::TeamDialogue => {
            println!("  {}", step.content);
        }
        StepType::TeamEscalation => {
            println!();
            print_colored("  ↑ Escalating: ", YELLOW);
            println!("{}", step.content);
        }
        // v0.2.9: Team dispatch and specialist working
        StepType::TeamDispatch => {
            print_colored("  ", DIM);
            println!("{}", step.content);
        }
        StepType::SpecialistWorking => {
            print_colored("  ", DIM);
            println_colored(&step.content, CYAN);
        }

        // DEBUG ONLY
        StepType::AnnaToLlm => {
            if debug {
                println_colored("  [prompt to LLM]", DIM);
            }
        }
        StepType::LlmCommands => {
            if debug {
                println_colored("  [LLM response]", DIM);
                if step.content != "NONE" && step.content != "DONE" {
                    for line in step.content.lines() {
                        let line = line.trim();
                        if !line.is_empty() {
                            print_colored("    $ ", DIM);
                            println_colored(line, CYAN);
                        }
                    }
                }
            }
        }
        StepType::CommandExec => {
            if debug {
                print_colored("  $ ", DIM);
                println!("{}", step.content);
            }
        }
        StepType::CommandOutput => {
            if debug {
                println_colored(&format!("  {}", step.content), DIM);
            }
        }
        StepType::ValidationPrompt | StepType::ValidationResponse | StepType::FinalPrompt => {
            if debug {
                println_colored("  [internal]", DIM);
            }
        }
        StepType::WikiSearch => {
            if debug {
                println_colored("  Checking Arch Wiki...", DIM);
            }
        }
        StepType::WikiResults | StepType::WikiCommands => {
            if debug {
                println_colored("  [wiki results]", DIM);
            }
        }
        StepType::IntentResult => {
            if debug {
                println_colored(&format!("  intent: {}", step.content), DIM);
            }
        }
        StepType::SubQuestion | StepType::SubQuestionResult => {
            if debug {
                println_colored(&format!("  {}", step.content), DIM);
            }
        }
    }
}

/// Print a single dialogue step (streaming mode)
pub fn print_step(step: &anna_shared::rpc::DialogueStep) {
    print_step_internal(step, false);
}

/// Print the full dialogue
#[allow(dead_code)]
pub fn print_dialogue(result: &AskResult) {
    for step in &result.dialogue {
        print_step_internal(step, true);
    }
}

/// Print timeout error
pub fn print_timeout_error(timeout_secs: u64) {
    println!();
    println_colored("REQUEST TIMED OUT", RED);
    println!();
    println!("  The request took longer than {}s.", timeout_secs);
    println!();
    println_colored("Possible causes:", YELLOW);
    println!("  - Ollama model is loading (first query is slow)");
    println!("  - Complex question requiring many iterations");
    println!("  - LLM server is overloaded");
    println!();
    println_colored("Try:", GREEN);
    println!("  - Run again - model may be loaded now");
    println!("  - Check: annactl status");
    println!();
}

/// Flush stdout
pub fn flush_stdout() {
    io::stdout().flush().ok();
}

/// Show proactive alerts from monitoring system
pub fn show_proactive_alerts() -> bool {
    let store = match IssueStore::load() {
        Ok(s) => s,
        Err(_) => return false,
    };

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

    if !critical.is_empty() {
        println_colored("Issues detected:", YELLOW);
        println!();
        for issue in &critical {
            print_colored("  ✗ ", RED);
            println!("{}", issue.summary);
            if let Some(ref fix) = issue.suggested_fix {
                println_colored(&format!("    → {}", fix), DIM);
            }
        }
    }

    if !warnings.is_empty() {
        if critical.is_empty() {
            println_colored("Heads up:", YELLOW);
            println!();
        }
        for issue in warnings.iter().take(3) {
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

/// Mark alerts as notified after showing them
pub fn mark_alerts_shown() {
    if let Ok(mut store) = IssueStore::load() {
        store.mark_notified();
        let _ = store.save();
    }
}

/// Print comprehensive stats
pub fn print_stats() {
    // v0.1.2: Use correct data directories
    // Memory is in ~/.anna/
    let anna_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".anna");
    // Tickets, XP are in ~/.local/share/anna/
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("anna");

    println!();
    println_colored("ANNA STATISTICS", BOLD);
    println!();

    // LEARNING
    println_colored("LEARNING", CYAN);

    let memory_path = anna_dir.join("memory.json");
    let (exp_count, pattern_count, cluster_count, memory_hits, memory_misses) = load_memory_stats(&memory_path);

    println!("  Experiences:   {}", exp_count);
    println!("  Patterns:      {}", pattern_count);
    println!("  Clusters:      {}", cluster_count);

    // Memory hit rate
    let total_queries = memory_hits + memory_misses;
    if total_queries > 0 {
        let hit_rate = memory_hits as f64 / total_queries as f64 * 100.0;
        print!("  Memory Hits:   ");
        let rate_color = if hit_rate >= 50.0 { GREEN } else if hit_rate >= 25.0 { YELLOW } else { DIM };
        print_colored(&format!("{:.1}%", hit_rate), rate_color);
        println_colored(&format!(" ({}/{})", memory_hits, total_queries), DIM);
    }
    println!();

    // PROGRESSION
    let xp_path = data_dir.join("xp.json");
    let (level, total_xp, title, progress, tickets_resolved) = load_xp_data(&xp_path);

    println_colored("PROGRESSION", CYAN);
    print!("  Level:         ");
    print_colored(&format!("{}", level), BOLD);
    print_colored(" / 100", DIM);
    print!("  ");
    println_colored(&format!("\"{}\"", title), YELLOW);

    print!("  XP:            {} ", total_xp);
    print_progress_bar(progress);
    println!();

    println!("  Tickets:       {} resolved", tickets_resolved);
    println!();

    // ACTIVITY
    println_colored("ACTIVITY", CYAN);

    let fix_history_path = data_dir.join("fix_history.json");
    let fixes_count = count_json_array(&fix_history_path, "fixes");
    println!("  Fixes Applied: {}", fixes_count);

    let deps_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".anna/installed_deps.txt");
    let helpers_count = if deps_path.exists() {
        std::fs::read_to_string(&deps_path).ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).count())
            .unwrap_or(0)
    } else { 0 };
    println!("  Helpers:       {} installed", helpers_count);
    println!();

    // TICKET METRICS
    let tickets_path = data_dir.join("tickets.json");
    if tickets_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&tickets_path) {
            if let Ok(store) = serde_json::from_str::<serde_json::Value>(&content) {
                println_colored("TICKETS", CYAN);

                let total_resolved = store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                let total_failed = store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0);

                println!("  Resolved:      {}", total_resolved);
                println!("  Failed:        {}", total_failed);

                if total_resolved > 0 {
                    let success_rate = total_resolved as f64 / (total_resolved + total_failed) as f64 * 100.0;
                    print!("  Success Rate:  ");
                    let rate_color = if success_rate >= 90.0 { GREEN } else if success_rate >= 70.0 { YELLOW } else { RED };
                    println_colored(&format!("{:.1}%", success_rate), rate_color);
                }
                println!();
            }
        }
    }
}

/// Load memory statistics from file
fn load_memory_stats(path: &std::path::Path) -> (usize, usize, usize, u64, u64) {
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(memory) = serde_json::from_str::<serde_json::Value>(&content) {
                let experiences = memory.get("experiences")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let patterns = memory.get("patterns")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let clusters = memory.get("clusters")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let stats = memory.get("stats");
                let hits = stats
                    .and_then(|s| s.get("memory_hits"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let misses = stats
                    .and_then(|s| s.get("memory_misses"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                return (experiences, patterns, clusters, hits, misses);
            }
        }
    }
    (0, 0, 0, 0, 0)
}

/// Load XP data from file
fn load_xp_data(path: &std::path::Path) -> (u32, u64, &'static str, f64, u64) {
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(xp) = serde_json::from_str::<serde_json::Value>(&content) {
                let level = xp.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let total = xp.get("total_xp").and_then(|v| v.as_u64()).unwrap_or(0);
                let tickets = xp.get("tickets_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                let title = get_title_for_level(level);
                let xp_needed = xp_for_level(level + 1);
                let xp_current = xp_for_level(level);
                let prog = if xp_needed > xp_current {
                    ((total.saturating_sub(xp_current)) as f64 / (xp_needed - xp_current) as f64 * 100.0).min(100.0)
                } else { 100.0 };
                return (level, total, title, prog, tickets);
            }
        }
    }
    (1, 0, "Helpdesk Newbie", 0.0, 0)
}

/// Count items in a JSON array field
fn count_json_array(path: &std::path::Path, field: &str) -> usize {
    if path.exists() {
        std::fs::read_to_string(path).ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|h| h.get(field).and_then(|f| f.as_array()).map(|a| a.len()))
            .unwrap_or(0)
    } else { 0 }
}

/// Print a simple progress bar
fn print_progress_bar(progress: f64) {
    let width = 20;
    let filled = (progress / 100.0 * width as f64) as usize;
    print_colored(&"=".repeat(filled), GREEN);
    print_colored(&"-".repeat(width - filled), DIM);
    print_colored(&format!(" {:.0}%", progress), GREEN);
}

/// Get title for level
fn get_title_for_level(level: u32) -> &'static str {
    match level {
        0..=5 => "Helpdesk Newbie",
        6..=10 => "Support Rookie",
        11..=15 => "Tech Apprentice",
        16..=20 => "Junior Analyst",
        21..=30 => "IT Assistant",
        31..=40 => "Tech Support Pro",
        41..=50 => "System Expert",
        51..=60 => "Tech Guru",
        61..=70 => "System Master",
        71..=80 => "Tech Wizard",
        81..=90 => "IT Sage",
        91..=99 => "System Overlord",
        100 => "The One Who Knows All",
        _ => "Unknown Entity",
    }
}

/// Calculate XP needed for level
fn xp_for_level(level: u32) -> u64 {
    let base = 100.0;
    let xp = base * (level as f64).powf(1.5) + (level as f64 * 50.0);
    xp as u64
}
