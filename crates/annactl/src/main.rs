//! Anna CLI (annactl) - User interface wrapper
//!
//! v4.1.0: Simplified Health - Detailed Dependencies, No Trust, Simple RPG
//!   - status shows: version, services, models per role, folders, success rate, level
//!   - Removed Trust metric - replaced with Success Rate
//!   - Shows background LLM activity (model downloads)
//!
//! v4.0.0: Debug Tracing, Reset Command, Learning Analytics
//!   - Enhanced debug mode with detailed message tracing and timing
//!   - `annactl reset` - factory reset command (for testing)
//!   - Learning analytics in stats (similar questions, delta improvements)
//!
//! v3.14.2: Split status/stats - status is quick health, stats is detailed
//! v3.14.0: Clean Status - single source of truth, no contradictions
//!
//! Architecture:
//!   - annactl is a THIN CLIENT - no LLM calls happen here
//!   - All questions route through the daemon API
//!   - Brain fast path handles simple questions locally (no LLM)
//!   - Help/version are static (no LLM needed)
//!
//! Allowed CLI surface (case-insensitive):
//!   - annactl                           Start interactive REPL
//!   - annactl <request>                 One-shot natural language request
//!   - annactl status                    Quick health check
//!   - annactl stats                     Detailed statistics + learning
//!   - annactl reset                     Factory reset (testing only)
//!   - annactl version                   Show version info
//!   - annactl -h | --help | help        Show help info

#![allow(dead_code)]
#![allow(unused_imports)]

mod ask_user;
mod client;
mod output;
mod progress_display;
mod spinner;
mod streaming_debug;

use anna_common::{
    clear_current_request, generate_request_id, init_logger, is_version_newer, log_request,
    logging, self_health, set_current_request, AnnaConfigV5, LogComponent,
    LogEntry, LogLevel, OverallHealth, RepairSafety, RequestContext, RequestStatus,
    StatsEngine, XpStore,
    try_fast_answer, telemetry_record_brain,
    XpEvent, XpEventType, FinalAnswer,
    debug_is_enabled, DebugIntent, DebugState, debug_set_enabled, debug_get_state,
    get_title_color, get_mood_text, get_streak_text, TrustLevel,
    record_brain_self_solve, UnifiedXpRecorder,
    brain_fast::{PendingActionType, execute_experience_reset, execute_factory_reset},
    is_benchmark_trigger,
    llm_provision::LlmSelection,
    format_percentage,
    THIN_SEPARATOR,
    xp_log::XpLog,
};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use std::io::{self, BufRead, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .init();

    let config = AnnaConfigV5::load();
    init_logger(config.log.clone());
    logging::logger().info(LogComponent::Request, "annactl starting");

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => run_repl().await,
        [cmd] if cmd.eq_ignore_ascii_case("status") => run_status().await,
        [cmd] if cmd.eq_ignore_ascii_case("stats") => run_stats().await,
        [cmd] if cmd.eq_ignore_ascii_case("reset") => run_reset().await,
        [flag] if flag == "-V" || flag == "--version" || flag.eq_ignore_ascii_case("version") => {
            run_version().await
        }
        [flag] if flag.eq_ignore_ascii_case("-h") || flag.eq_ignore_ascii_case("--help") || flag.eq_ignore_ascii_case("help") => {
            run_help()
        }
        [question] if output::handle_explain_request(question) => Ok(()),
        [question] => run_ask(question).await,
        words => {
            let question = words.join(" ");
            if output::handle_explain_request(&question) {
                Ok(())
            } else {
                run_ask(&question).await
            }
        }
    }
}

/// Pending action for confirmation flow
#[derive(Debug, Clone, PartialEq)]
enum PendingAction {
    None,
    Reset(PendingActionType),
}

/// Run the interactive REPL
async fn run_repl() -> Result<()> {
    print_banner();
    run_startup_health_check();

    println!("{}  Interactive mode. Type {} to exit.\n", ">>".cyan(), "quit".yellow());

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut pending_action = PendingAction::None;

    loop {
        print!("{}  ", "anna>".bright_magenta());
        stdout.flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q" | ":q") {
            println!("\nGoodbye.\n");
            break;
        }

        // Handle pending confirmation
        if let PendingAction::Reset(ref action_type) = pending_action {
            if action_type.is_confirmed(input) {
                let result = match action_type {
                    PendingActionType::ExperienceReset => execute_experience_reset(),
                    PendingActionType::FactoryReset => execute_factory_reset(),
                };
                println!();
                print_final_answer(&result.text, result.reliability, &result.origin, result.duration_ms);
                println!();
            } else {
                let action_name = match action_type {
                    PendingActionType::ExperienceReset => "Experience reset",
                    PendingActionType::FactoryReset => "Factory reset",
                };
                println!();
                println!("  {}  {} cancelled.", "x".bright_red(), action_name);
                println!();
            }
            pending_action = PendingAction::None;
            continue;
        }

        // Handle special commands in REPL
        if input == "-V" || input == "--version" || input.eq_ignore_ascii_case("version") {
            run_version().await?;
            continue;
        }
        if matches!(input.to_lowercase().as_str(), "help" | "-h" | "--help") {
            run_help()?;
            continue;
        }
        if input.eq_ignore_ascii_case("status") {
            run_status().await?;
            continue;
        }
        if input.eq_ignore_ascii_case("stats") {
            run_stats().await?;
            continue;
        }
        if input.eq_ignore_ascii_case("reset") {
            run_reset().await?;
            continue;
        }
        if output::handle_explain_request(input) {
            continue;
        }

        // Process question
        match run_ask_with_pending(input).await {
            Ok(Some(action)) => pending_action = action,
            Ok(None) => {}
            Err(e) => eprintln!("[ERROR] {}", e),
        }
    }

    Ok(())
}

fn print_banner() {
    println!(
        "\n{}  {}",
        ">>".bright_magenta(),
        format!("Anna v{}", env!("CARGO_PKG_VERSION")).bright_white().bold()
    );
    println!("   Your intelligent Linux assistant\n");
}

/// Ask Anna a question
async fn run_ask(question: &str) -> Result<()> {
    run_ask_with_pending(question).await.map(|_| ())
}

/// Ask Anna a question - returns pending action for REPL
async fn run_ask_with_pending(question: &str) -> Result<Option<PendingAction>> {
    let request_id = generate_request_id();
    let sanitized_query = logging::sanitize_query(question);
    let ctx = RequestContext::new(request_id.clone(), sanitized_query.clone());
    set_current_request(ctx);

    // Handle debug mode control
    let debug_intent = DebugIntent::classify(question);
    if debug_intent.is_debug_intent() {
        let response = handle_debug_intent(debug_intent);
        spinner::print_question(question);
        println!();
        print_final_answer(&response, 1.0, "Brain", 1);
        clear_current_request();
        return Ok(None);
    }

    // Try Brain fast path
    if let Some(fast_answer) = try_fast_answer(question) {
        if is_benchmark_trigger(&fast_answer) {
            // Fall through to daemon for benchmarks
        } else {
            spinner::print_question(question);
            println!();

            if fast_answer.pending_confirmation {
                print_final_answer(&fast_answer.text, fast_answer.reliability, &fast_answer.origin, fast_answer.duration_ms);
                println!();
                clear_current_request();
                if let Some(action_type) = fast_answer.pending_action {
                    return Ok(Some(PendingAction::Reset(action_type)));
                }
                return Ok(None);
            }

            print_final_answer(&fast_answer.text, fast_answer.reliability, &fast_answer.origin, fast_answer.duration_ms);
            telemetry_record_brain(question, fast_answer.reliability, fast_answer.duration_ms);

            let xp_line = record_brain_self_solve(question, &fast_answer.origin);
            if streaming_debug::is_debug_enabled() {
                println!("{}", xp_line);
            } else {
                println!("{}", xp_line.dimmed());
            }

            clear_current_request();
            return Ok(None);
        }
    }

    // Debug streaming mode
    if streaming_debug::is_debug_enabled() {
        run_ask_with_debug_stream(question, &request_id).await?;
        return Ok(None);
    }

    spinner::print_question(question);
    let daemon = client::DaemonClient::new();

    if !daemon.is_healthy().await {
        eprintln!("[ERROR] Anna daemon is not running");
        eprintln!("   Run: {} to start", "sudo systemctl start annad".cyan());
        clear_current_request();
        std::process::exit(1);
    }

    let thinking = spinner::Spinner::new("thinking...");

    match daemon.answer(question).await {
        Ok(final_answer) => {
            process_llm_xp_events(question, &final_answer);
            clear_current_request();
            let elapsed = thinking.finish();

            if std::env::var("ANNA_QA_MODE").is_ok() {
                let qa_output = final_answer.to_qa_output();
                println!("{}", serde_json::to_string_pretty(&qa_output).unwrap_or_default());
            } else {
                output::display_final_answer_v100(&final_answer, elapsed);
            }
            Ok(None)
        }
        Err(e) => {
            thinking.stop();
            clear_current_request();
            output::display_error(&format!("Failed to get answer: {}", e));
            Err(e)
        }
    }
}

/// Ask with live debug streaming
async fn run_ask_with_debug_stream(question: &str, _request_id: &str) -> Result<()> {
    let daemon = client::DaemonClient::new();

    if !daemon.is_healthy().await {
        eprintln!("[ERROR] Anna daemon is not running");
        eprintln!("   Run: {} to start", "sudo systemctl start annad".cyan());
        clear_current_request();
        std::process::exit(1);
    }

    match streaming_debug::stream_answer_with_debug(question).await {
        Ok(_) => {
            clear_current_request();
            Ok(())
        }
        Err(e) => {
            clear_current_request();
            output::display_error(&format!("Debug stream failed: {}", e));
            Err(e)
        }
    }
}

/// Version display
async fn run_version() -> Result<()> {
    let daemon = client::DaemonClient::new();

    println!();
    println!("{}", "ANNA VERSION".bright_white().bold());
    println!("{}", THIN_SEPARATOR);
    println!("  {}  annactl v{}", "*".cyan(), env!("CARGO_PKG_VERSION"));

    if daemon.is_healthy().await {
        if let Ok(health) = daemon.health().await {
            println!(
                "  {}  annad v{} (uptime {})",
                "+".bright_green(),
                health.version,
                format_uptime(health.uptime_seconds).cyan()
            );
        }
    } else {
        println!("  {}  annad not running", "!".bright_red());
    }

    if let Some(ver) = check_ollama_version() {
        println!("  {}  ollama {}", "+".bright_green(), ver);
    } else {
        println!("  {}  ollama not available", "!".bright_red());
    }

    println!("{}", THIN_SEPARATOR);
    println!();
    Ok(())
}

fn check_ollama_version() -> Option<String> {
    use std::process::Command;
    let output = Command::new("ollama").args(["--version"]).output().ok()?;
    if output.status.success() {
        let ver = String::from_utf8_lossy(&output.stdout);
        let ver = ver.trim();
        if let Some(v) = ver.strip_prefix("ollama version is ") {
            Some(format!("v{}", v))
        } else {
            Some(ver.to_string())
        }
    } else {
        None
    }
}

/// Help display
fn run_help() -> Result<()> {
    println!();
    println!("{}", "ANNA HELP".bright_white().bold());
    println!("{}", THIN_SEPARATOR);
    println!();
    println!("{}", "[COMMANDS]".cyan());
    println!("  annactl                   Interactive REPL");
    println!("  annactl \"<question>\"      Ask Anna anything");
    println!("  annactl status            Quick health check");
    println!("  annactl stats             Detailed statistics");
    println!("  annactl reset             {} (testing)", "Factory reset".bright_red());
    println!("  annactl version           Version info");
    println!("  annactl help              This help");
    println!();
    println!("{}", "[DEBUG MODE]".cyan());
    println!("  \"enable debug mode\"       Show detailed tracing");
    println!("  \"disable debug mode\"      Normal operation");
    println!();
    println!("{}", "[EXAMPLES]".cyan());
    println!("  annactl \"How many CPU cores?\"");
    println!("  annactl \"What's my RAM usage?\"");
    println!();
    println!("{}", THIN_SEPARATOR);
    println!();
    Ok(())
}

// ============================================================================
// STATUS COMMAND - v4.1.0: Detailed health, no trust, simplified RPG
// ============================================================================
// Color legend:
//   GREEN  = healthy/running/good
//   YELLOW = degraded/warning
//   RED    = down/error/critical
//   CYAN   = informational
//   DIM    = context/secondary

/// Status command - Anna's health and system status
async fn run_status() -> Result<()> {
    let daemon = client::DaemonClient::new();

    println!();
    println!("{}", "ANNA STATUS".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // VERSION section
    println!("{}", "[VERSION]".cyan());
    println!("  annactl:    v{}", env!("CARGO_PKG_VERSION"));

    let daemon_running = daemon.is_healthy().await;
    if daemon_running {
        if let Ok(health) = daemon.health().await {
            println!("  annad:      v{}", health.version);
        }
    }

    // SERVICES section
    println!();
    println!("{}", "[SERVICES]".cyan());

    // Daemon
    if daemon_running {
        if let Ok(health) = daemon.health().await {
            println!(
                "  Daemon:     {} {}",
                "running".bright_green(),
                format!("(up {})", format_uptime(health.uptime_seconds)).dimmed()
            );
        } else {
            println!("  Daemon:     {}", "running".bright_green());
        }
    } else {
        println!("  Daemon:     {}", "DOWN".bright_red());
    }

    // Ollama
    let llm_running = check_llm_running();
    if llm_running {
        if let Some(activity) = check_ollama_activity() {
            println!("  Ollama:     {} {}", "running".bright_green(), format!("({})", activity).yellow());
        } else {
            println!("  Ollama:     {}", "running".bright_green());
        }
    } else {
        println!("  Ollama:     {}", "DOWN".bright_red());
    }

    // MODELS section - show each role's model
    println!();
    println!("{}", "[MODELS]".cyan());

    let selection = LlmSelection::load();

    if selection.autoprovision_status == "completed" {
        // Router
        let router_ok = check_model_installed(&selection.router_model);
        if router_ok {
            println!("  Router:     {} {}", selection.router_model.bright_green(), format!("({}%)", (selection.router_score * 100.0).round() as i32).dimmed());
        } else {
            println!("  Router:     {} {}", selection.router_model.bright_red(), "NOT INSTALLED".bright_red());
        }

        // Junior
        let junior_ok = check_model_installed(&selection.junior_model);
        if junior_ok {
            println!("  Junior:     {} {}", selection.junior_model.bright_green(), format!("({}%)", (selection.junior_score * 100.0).round() as i32).dimmed());
        } else {
            println!("  Junior:     {} {}", selection.junior_model.bright_red(), "NOT INSTALLED".bright_red());
        }

        // Senior
        let senior_ok = check_model_installed(&selection.senior_model);
        if senior_ok {
            println!("  Senior:     {} {}", selection.senior_model.bright_green(), format!("({}%)", (selection.senior_score * 100.0).round() as i32).dimmed());
        } else {
            println!("  Senior:     {} {}", selection.senior_model.bright_red(), "NOT INSTALLED".bright_red());
        }

        // Last benchmark
        if !selection.last_benchmark.is_empty() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&selection.last_benchmark) {
                let local = dt.with_timezone(&chrono::Local);
                println!("  Configured: {}", local.format("%Y-%m-%d %H:%M").to_string().dimmed());
            }
        }
    } else if selection.autoprovision_status.is_empty() {
        println!("  {}", "Not configured - run benchmark to auto-provision".yellow());
    } else {
        println!("  {}", format!("Provisioning: {}", selection.autoprovision_status).yellow());
    }

    // FOLDERS section
    println!();
    println!("{}", "[FOLDERS]".cyan());

    let folders = [
        ("/var/lib/anna", "Data directory"),
        ("/var/lib/anna/xp", "XP storage"),
        ("/var/lib/anna/knowledge", "Knowledge base"),
        ("/var/lib/anna/llm", "LLM configs"),
    ];

    for (path, desc) in folders {
        let exists = std::path::Path::new(path).exists();
        let writable = std::fs::metadata(path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false);

        if exists && writable {
            println!("  {}  {} {}", "+".bright_green(), path, desc.dimmed());
        } else if exists {
            println!("  {}  {} {} {}", "!".yellow(), path, desc.dimmed(), "(read-only)".yellow());
        } else {
            println!("  {}  {} {} {}", "x".bright_red(), path, desc.dimmed(), "(missing)".bright_red());
        }
    }

    // PERFORMANCE section - success rate instead of trust
    println!();
    println!("{}", "[PERFORMANCE]".cyan());

    let xp_store = XpStore::load();
    let total_good = xp_store.anna.total_good;
    let total_bad = xp_store.anna.total_bad;
    let total = total_good + total_bad;

    if total > 0 {
        let success_rate = (total_good as f64 / total as f64 * 100.0).round() as i32;
        let rate_colored = if success_rate >= 70 {
            format!("{}%", success_rate).bright_green().to_string()
        } else if success_rate >= 50 {
            format!("{}%", success_rate).yellow().to_string()
        } else {
            format!("{}%", success_rate).bright_red().to_string()
        };
        println!("  Success:    {} ({}/{})", rate_colored, total_good, total);
    } else {
        println!("  Success:    {} (no data)", "---".dimmed());
    }

    // PROGRESSION section - simplified RPG
    println!();
    println!("{}", "[PROGRESSION]".cyan());

    let level = xp_store.anna.level;
    let title = match level {
        0 => "Novice",
        1 => "Apprentice",
        2 => "Assistant",
        3 => "Specialist",
        4 => "Expert",
        5 => "Master",
        _ => "Grandmaster",
    };
    println!("  Level:      {} {}", level, format!("({})", title).dimmed());

    let xp = xp_store.anna.xp;
    let xp_to_next = xp_store.anna.xp_to_next;
    let xp_pct = if xp_to_next > 0 { (xp as f64 / xp_to_next as f64 * 100.0).round() as i32 } else { 0 };
    println!("  XP:         {}/{} ({}%)", xp, xp_to_next, xp_pct);

    println!("{}", THIN_SEPARATOR);

    // Debug mode
    if debug_is_enabled() {
        println!("  {}", "[DEBUG MODE ACTIVE]".bright_cyan());
        println!();
    }

    Ok(())
}

/// Check if a model is installed in ollama
fn check_model_installed(model: &str) -> bool {
    use std::process::Command;

    if let Ok(output) = Command::new("ollama").args(["list"]).output() {
        let list = String::from_utf8_lossy(&output.stdout);
        // Model names can be "qwen2.5:3b-instruct" - check if base name matches
        list.lines().any(|line| line.contains(model.split(':').next().unwrap_or(model)))
    } else {
        false
    }
}

/// Check if ollama is actively pulling/downloading a model
fn check_ollama_activity() -> Option<String> {
    use std::process::Command;

    // Check for ollama pull processes
    if let Ok(output) = Command::new("pgrep").args(["-a", "ollama"]).output() {
        let processes = String::from_utf8_lossy(&output.stdout);
        for line in processes.lines() {
            if line.contains("pull") {
                // Extract model name from "ollama pull model:tag"
                if let Some(idx) = line.find("pull") {
                    let rest = &line[idx + 4..].trim();
                    let model = rest.split_whitespace().next().unwrap_or("unknown");
                    return Some(format!("Downloading: {}", model));
                }
            }
        }
    }
    None
}

// ============================================================================
// STATS COMMAND - v4.0.0: User + LLM + Anna statistics + Learning Analytics
// ============================================================================

/// Stats command - all statistics in one view
async fn run_stats() -> Result<()> {
    println!();
    println!("{}", "ANNA STATS".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    let xp_store = XpStore::load();
    let selection = LlmSelection::load();
    let xp_log = XpLog::new();

    // USER INTERACTIONS
    println!("{}", "[USER INTERACTIONS]".cyan());

    let total_good = xp_store.anna.total_good;
    let total_bad = xp_store.anna.total_bad;
    let total = total_good + total_bad;

    if total > 0 {
        println!("  Total:      {}", total);
        println!("  Successful: {} {}", total_good.to_string().bright_green(), format!("({}%)", (total_good as f64 / total as f64 * 100.0).round() as i32).dimmed());
        println!("  Failed:     {} {}", total_bad.to_string().bright_red(), format!("({}%)", (total_bad as f64 / total as f64 * 100.0).round() as i32).dimmed());

        // Streak
        if xp_store.anna.streak_good > 0 {
            println!("  Streak:     {} {}", xp_store.anna.streak_good.to_string().bright_green(), "good".dimmed());
        } else if xp_store.anna.streak_bad > 0 {
            println!("  Streak:     {} {}", xp_store.anna.streak_bad.to_string().bright_red(), "bad".dimmed());
        }
    } else {
        println!("  No interactions yet");
    }

    // ANSWER ORIGIN
    println!();
    println!("{}", "[ANSWER ORIGIN]".cyan());

    let brain = xp_store.anna_stats.self_solves;
    let llm = xp_store.anna_stats.llm_answers;
    let timeouts = xp_store.anna_stats.timeouts;
    let origin_total = brain + llm;

    if origin_total > 0 {
        let brain_pct = (brain as f64 / origin_total as f64 * 100.0).round() as i32;
        let llm_pct = (llm as f64 / origin_total as f64 * 100.0).round() as i32;
        println!("  Brain:      {} ({}%)", brain, brain_pct);
        println!("  LLM:        {} ({}%)", llm, llm_pct);
    } else {
        println!("  Brain:      {}", brain);
        println!("  LLM:        {}", llm);
    }
    if timeouts > 0 {
        println!("  Timeouts:   {}", timeouts.to_string().bright_red());
    }

    // LLM PERFORMANCE
    println!();
    println!("{}", "[LLM PERFORMANCE]".cyan());

    // Junior
    let junior_total = xp_store.junior_stats.good_plans + xp_store.junior_stats.bad_plans;
    let junior_rate = if junior_total > 0 {
        (xp_store.junior_stats.good_plans as f64 / junior_total as f64 * 100.0).round() as i32
    } else { 0 };

    println!("  Junior:");
    println!("    Good:     {}", xp_store.junior_stats.good_plans.to_string().bright_green());
    println!("    Bad:      {}", xp_store.junior_stats.bad_plans.to_string().bright_red());
    if junior_total > 0 {
        let rate_colored = if junior_rate >= 70 {
            format!("{}%", junior_rate).bright_green().to_string()
        } else if junior_rate >= 50 {
            format!("{}%", junior_rate).yellow().to_string()
        } else {
            format!("{}%", junior_rate).bright_red().to_string()
        };
        println!("    Success:  {}", rate_colored);
    }

    // Senior
    let senior_total = xp_store.senior_stats.approvals + xp_store.senior_stats.fix_and_accept + xp_store.senior_stats.refusals;
    let senior_rate = if senior_total > 0 {
        ((xp_store.senior_stats.approvals + xp_store.senior_stats.fix_and_accept) as f64 / senior_total as f64 * 100.0).round() as i32
    } else { 0 };

    println!("  Senior:");
    println!("    Approved: {}", xp_store.senior_stats.approvals.to_string().bright_green());
    println!("    Fixed:    {}", xp_store.senior_stats.fix_and_accept.to_string().yellow());
    println!("    Refused:  {}", xp_store.senior_stats.refusals.to_string().bright_red());
    if senior_total > 0 {
        let rate_colored = if senior_rate >= 70 {
            format!("{}%", senior_rate).bright_green().to_string()
        } else if senior_rate >= 50 {
            format!("{}%", senior_rate).yellow().to_string()
        } else {
            format!("{}%", senior_rate).bright_red().to_string()
        };
        println!("    Success:  {}", rate_colored);
    }

    // LLM MODELS
    println!();
    println!("{}", "[LLM MODELS]".cyan());

    if selection.autoprovision_status == "completed" {
        if let Some(ref tier) = selection.hardware_tier {
            println!("  Tier:       {:?}", tier);
        }

        let router_score = (selection.router_score * 100.0).round() as i32;
        let junior_score = (selection.junior_score * 100.0).round() as i32;
        let senior_score = (selection.senior_score * 100.0).round() as i32;

        println!("  Router:     {} ({}%)", selection.router_model, router_score);
        println!("  Junior:     {} ({}%)", selection.junior_model, junior_score);
        println!("  Senior:     {} ({}%)", selection.senior_model, senior_score);

        if !selection.last_benchmark.is_empty() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&selection.last_benchmark) {
                let local = dt.with_timezone(&chrono::Local);
                println!("  Benchmark:  {}", local.format("%Y-%m-%d %H:%M"));
            }
        }
    } else if selection.autoprovision_status.is_empty() {
        println!("  Not configured");
    } else {
        println!("  {}", selection.autoprovision_status.bright_red());
    }

    // LEARNING (24h metrics)
    println!();
    println!("{}", "[LEARNING (24h)]".cyan());

    let metrics = xp_log.metrics_24h();

    if metrics.total_events > 0 {
        // Net XP change
        let net_colored = if metrics.net_xp >= 0 {
            format!("+{}", metrics.net_xp).bright_green().to_string()
        } else {
            format!("{}", metrics.net_xp).bright_red().to_string()
        };
        println!("  XP Net:     {} (gained {} / lost {})", net_colored, metrics.xp_gained, metrics.xp_lost);
        println!("  Events:     {} (+{} / -{})", metrics.total_events, metrics.positive_events, metrics.negative_events);
        println!("  Questions:  {}", metrics.questions_answered);

        // Top event types
        if let Some(ref top_pos) = metrics.top_positive {
            println!("  Best:       {}", top_pos.bright_green());
        }
        if let Some(ref top_neg) = metrics.top_negative {
            println!("  Worst:      {}", top_neg.bright_red());
        }
    } else {
        println!("  No learning events in past 24h");
    }

    // Recent XP events (last 5)
    let recent = xp_log.read_recent(5);
    if !recent.is_empty() {
        println!();
        println!("{}", "[RECENT EVENTS]".cyan());
        for event in recent.iter().take(5) {
            let xp_str = if event.xp_change >= 0 {
                format!("+{}", event.xp_change).bright_green().to_string()
            } else {
                format!("{}", event.xp_change).bright_red().to_string()
            };
            // Truncate question to 30 chars
            let q: String = event.question.chars().take(30).collect();
            let q_display = if event.question.len() > 30 {
                format!("{}...", q)
            } else {
                q
            };
            println!("  {} {:>4}  {}", event.event_type.dimmed(), xp_str, q_display.dimmed());
        }
    }

    println!("{}", THIN_SEPARATOR);
    println!();
    Ok(())
}

fn check_llm_running() -> bool {
    use std::process::Command;

    // Try systemctl first
    if let Ok(output) = Command::new("systemctl").args(["is-active", "ollama"]).output() {
        if String::from_utf8_lossy(&output.stdout).trim() == "active" {
            return true;
        }
    }

    // Try pgrep
    if let Ok(output) = Command::new("pgrep").arg("ollama").output() {
        if output.status.success() {
            return true;
        }
    }

    false
}

fn format_uptime(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    if seconds < MINUTE {
        format!("{}s", seconds)
    } else if seconds < HOUR {
        let mins = seconds / MINUTE;
        let secs = seconds % MINUTE;
        if secs > 0 { format!("{}m {}s", mins, secs) } else { format!("{}m", mins) }
    } else if seconds < DAY {
        let hours = seconds / HOUR;
        let mins = (seconds % HOUR) / MINUTE;
        if mins > 0 { format!("{}h {}m", hours, mins) } else { format!("{}h", hours) }
    } else {
        let days = seconds / DAY;
        let hours = (seconds % DAY) / HOUR;
        if hours > 0 { format!("{}d {}h", days, hours) } else { format!("{}d", days) }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn run_startup_health_check() {
    let report = self_health::run_with_auto_repair();

    match report.overall {
        OverallHealth::Healthy => {}
        OverallHealth::Degraded => {
            println!("{}  Self-health: {}", "[NOTE]".yellow(), "degraded".yellow());
            for component in &report.components {
                if !component.status.is_healthy() {
                    println!("   * {}: {}", component.name.yellow(), component.message);
                }
            }
            println!();
        }
        OverallHealth::Critical => {
            println!("{}  Self-health: {}", "[WARNING]".bright_red(), "critical".bright_red());
            for component in &report.components {
                if !component.status.is_healthy() {
                    println!("   * {}: {}", component.name.bright_red(), component.message);
                }
            }
            println!();
        }
        OverallHealth::Unknown => {
            println!("{}  Self-health: {}", "[NOTE]".dimmed(), "unknown".dimmed());
            println!();
        }
    }

    if !report.repairs_executed.is_empty() {
        println!("{}  Auto-repairs executed:", "[AUTO-REPAIR]".bright_green());
        for repair in &report.repairs_executed {
            let status = if repair.success { "+".bright_green().to_string() } else { "!".bright_red().to_string() };
            println!("   {} {}", status, repair.message);
        }
        println!();
    }
}

fn print_final_answer(text: &str, reliability: f64, origin: &str, duration_ms: u64) {
    println!();
    println!("{}", THIN_SEPARATOR);
    println!("{}", "Anna".bright_white().bold());
    println!("{}", THIN_SEPARATOR);
    println!();
    println!("{}", text);
    println!();
    println!("{}", THIN_SEPARATOR);

    let rel_pct = format_percentage(reliability);
    let rel_label = if reliability >= 0.9 {
        format!("{} ({})", rel_pct.bright_green(), "Green".bright_green())
    } else if reliability >= 0.7 {
        format!("{} ({})", rel_pct.yellow(), "Yellow".yellow())
    } else {
        format!("{} ({})", rel_pct.bright_red(), "Red".bright_red())
    };
    println!("Reliability: {}", rel_label);
    println!("Origin: {}", origin.cyan());

    let dur_str = if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else {
        format!("{:.2}s", duration_ms as f64 / 1000.0)
    };
    println!("Duration: {}", dur_str);
    println!("{}", THIN_SEPARATOR);
    println!();
}

fn handle_debug_intent(intent: DebugIntent) -> String {
    match intent {
        DebugIntent::Enable => {
            if let Err(e) = debug_set_enabled(true, "user_command") {
                return format!("Failed to enable debug mode: {}", e);
            }
            DebugState::format_enable_message()
        }
        DebugIntent::Disable => {
            if let Err(e) = debug_set_enabled(false, "user_command") {
                return format!("Failed to disable debug mode: {}", e);
            }
            DebugState::format_disable_message()
        }
        DebugIntent::Status => {
            let state = debug_get_state();
            state.format_status()
        }
        DebugIntent::None => "Not a debug command.".to_string(),
    }
}

/// Process XP events from a FinalAnswer
fn process_llm_xp_events(question: &str, answer: &FinalAnswer) {
    let recorder = UnifiedXpRecorder::new();
    let senior_verdict = answer.senior_verdict.as_deref().unwrap_or("unknown");
    let junior_had_draft = answer.junior_had_draft;
    let confidence = answer.scores.overall;

    // Junior XP
    if junior_had_draft && (senior_verdict == "approve" || senior_verdict == "fix_and_accept") {
        let xp_line = recorder.junior_clean_proposal(question, "");
        if streaming_debug::is_debug_enabled() {
            println!("{}", xp_line);
        }
    } else if !junior_had_draft || senior_verdict == "refuse" {
        let xp_line = recorder.junior_bad_command(question, "", &format!("verdict={}", senior_verdict));
        if streaming_debug::is_debug_enabled() {
            println!("{}", xp_line);
        }
    }

    // Senior XP
    match senior_verdict {
        "approve" if confidence >= 0.9 => {
            let xp_line = recorder.senior_green_approval(question, confidence);
            if streaming_debug::is_debug_enabled() {
                println!("{}", xp_line);
            }
        }
        "fix_and_accept" => {
            let xp_line = recorder.senior_fix_and_accept(question);
            if streaming_debug::is_debug_enabled() {
                println!("{}", xp_line);
            }
        }
        "refuse" => {
            let xp_line = recorder.low_reliability_refusal(question);
            if streaming_debug::is_debug_enabled() {
                println!("{}", xp_line);
            }
        }
        _ => {
            if confidence >= 0.7 {
                let xp_line = recorder.senior_green_approval(question, confidence);
                if streaming_debug::is_debug_enabled() {
                    println!("{}", xp_line);
                }
            }
        }
    }

    // Handle failures
    if let Some(ref failure_cause) = answer.failure_cause {
        match failure_cause.as_str() {
            "timeout_or_latency" => {
                let xp_line = recorder.llm_timeout(question, 0);
                if streaming_debug::is_debug_enabled() {
                    println!("{}", xp_line);
                }
            }
            "unsupported_domain" => {
                let xp_line = recorder.low_reliability_refusal(question);
                if streaming_debug::is_debug_enabled() {
                    println!("{}", xp_line);
                }
            }
            _ => {}
        }
    }
}

// ============================================================================
// RESET COMMAND - v4.0.0: Factory reset (testing only)
// ============================================================================

/// Reset command - complete factory reset (for testing)
async fn run_reset() -> Result<()> {
    println!();
    println!("{}", "ANNA RESET".bright_white().bold());
    println!("{}", THIN_SEPARATOR);
    println!();

    // Show warning
    println!("{}  This will delete ALL data:", "[WARNING]".bright_red());
    println!("   - XP and progression");
    println!("   - Knowledge and facts");
    println!("   - LLM configurations");
    println!("   - Benchmarks and stats");
    println!("   - Telemetry and logs");
    println!();

    // Execute factory reset directly
    let result = execute_factory_reset();

    if result.reliability >= 0.9 {
        println!("{}  Reset complete", "[OK]".bright_green());
        println!();
        println!("{}", result.text);
    } else {
        println!("{}  Reset failed", "[ERROR]".bright_red());
        println!();
        println!("{}", result.text);
    }

    println!();
    println!("{}", THIN_SEPARATOR);
    println!();
    Ok(())
}
