//! Anna CLI (annactl) - User interface wrapper
//!
//! v0.3.0: Strict CLI with LLM-orchestrated help/version
//! v0.4.0: Update status in version/help output
//! v0.5.0: Natural language configuration, hardware-aware model selection
//! v0.6.0: ASCII-only sysadmin style, multi-round reliability refinement
//! v0.7.0: Self-health monitoring with auto-repair and REPL notifications
//! v0.8.0: Observability and debug logging with JSONL output
//! v0.9.0: Version-aware installer, fully automatic auto-update, status command
//! v0.10.0: Evidence-based answers with LLM-A/LLM-B audit loop
//!
//! Allowed CLI surface (case-insensitive):
//!   - annactl                           Start interactive REPL
//!   - annactl <request>                 One-shot natural language request
//!   - annactl status                    Compact status summary
//!   - annactl version                   Show version info
//!   - annactl -h | --help | help        Show help info

// Allow dead code for features planned but not yet fully wired
#![allow(dead_code)]
#![allow(unused_imports)]

mod ask_user;
mod client;
mod llm_client;
mod orchestrator;
mod output;
mod progress_display;
mod spinner;
mod streaming_debug;

use anna_common::{
    clear_current_request, generate_request_id, init_logger, is_version_newer, log_request,
    logging, self_health, set_current_request, AnnaConfigV5, HardwareProfile, LogComponent,
    LogEntry, LogLevel, OverallHealth, RepairSafety, RequestContext, RequestStatus,
    StatsEngine, XpLog,
    // v0.86.0: XP tracking
    XpStore,
    // v0.87.0: Brain fast path
    try_fast_answer, FastQuestionType,
    // v0.88.0: XP events for Junior/Senior
    XpEvent, XpEventType, FinalAnswer,
    // v0.89.0: Persistent debug mode
    debug_is_enabled,
    // v0.91.0: Natural language debug control
    DebugIntent, DebugState, debug_set_enabled, debug_get_state,
    // v0.92.0: Decision Policy
    DecisionPolicy,
};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false),
        )
        .init();

    // v0.8.0: Initialize logging subsystem
    let config = AnnaConfigV5::load();
    init_logger(config.log.clone());

    // Log startup
    logging::logger().info(LogComponent::Request, "annactl starting");

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        // No arguments - start REPL
        [] => run_repl().await,

        // v0.9.0: Status command (case-insensitive)
        [cmd] if cmd.eq_ignore_ascii_case("status") => run_status().await,

        // v0.18.0: Version flags and command - instant, no daemon required
        [flag]
            if flag == "-V"
                || flag == "--version"
                || flag.eq_ignore_ascii_case("version") =>
        {
            run_version_instant().await
        }

        // Help flags - route through LLM (case-insensitive)
        [flag]
            if flag.eq_ignore_ascii_case("-h")
                || flag.eq_ignore_ascii_case("--help")
                || flag.eq_ignore_ascii_case("help") =>
        {
            run_help_via_llm().await
        }

        // Single quoted question
        [question] => run_ask(question).await,

        // Multiple words as question
        words => {
            let question = words.join(" ");
            run_ask(&question).await
        }
    }
}

/// Run the interactive REPL
async fn run_repl() -> Result<()> {
    print_banner();

    // v0.7.0: Run self-health check with auto-repair on startup
    run_startup_health_check();

    // v0.6.0: ASCII-only output
    println!(
        "{}  Interactive mode. Type {} to exit.\n",
        ">>".cyan(),
        "quit".yellow()
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Prompt
        print!("{}  ", "anna>".bright_magenta());
        stdout.flush()?;

        // Read input
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            // EOF
            break;
        }

        let input = line.trim();

        // Handle exit commands
        if input.is_empty() {
            continue;
        }
        if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q" | ":q") {
            // v0.6.0: ASCII-only output
            println!("\nGoodbye.\n");
            break;
        }

        // Handle version/help/status in REPL too (case-insensitive)
        // v0.18.0: Support -V/--version flags and "version" word
        if input == "-V"
            || input == "--version"
            || input.eq_ignore_ascii_case("version")
        {
            run_version_instant().await?;
            continue;
        }
        if matches!(input.to_lowercase().as_str(), "help" | "-h" | "--help") {
            run_help_via_llm().await?;
            continue;
        }
        if input.eq_ignore_ascii_case("status") {
            run_status().await?;
            continue;
        }

        // Process question
        if let Err(e) = run_ask(input).await {
            eprintln!("[ERROR] {}", e);
        }
    }

    Ok(())
}

fn print_banner() {
    // v0.6.0: ASCII-only banner
    println!(
        "\n{}  {}",
        ">>".bright_magenta(),
        format!("Anna v{}", env!("CARGO_PKG_VERSION"))
            .bright_white()
            .bold()
    );
    println!("   Your intelligent Linux assistant\n");
}

/// Ask Anna a question - the core function
/// v0.10.0: Uses evidence-based answer engine with LLM-A/LLM-B audit loop
/// v0.43.0: Added live debug streaming with ANNA_DEBUG=1
async fn run_ask(question: &str) -> Result<()> {
    // v0.8.0: Create request context with correlation ID
    let request_id = generate_request_id();
    let sanitized_query = logging::sanitize_query(question);
    let ctx = RequestContext::new(request_id.clone(), sanitized_query.clone());
    set_current_request(ctx);

    // Log request start
    logging::logger().write_daemon(
        &LogEntry::new(
            LogLevel::Debug,
            LogComponent::Request,
            "Processing question",
        )
        .with_request_id(&request_id)
        .with_fields(serde_json::json!({
            "query_length": question.len(),
            "query_preview": if question.len() > 50 { &question[..50] } else { question }
        })),
    );

    // v0.91.0: Handle debug mode control via natural language (no LLM needed)
    // This must come BEFORE try_fast_answer to intercept debug commands
    let debug_intent = DebugIntent::classify(question);
    if debug_intent.is_debug_intent() {
        let response = handle_debug_intent(question, debug_intent);
        spinner::print_question(question);
        println!();
        print_final_answer(&response, 1.0, "Brain", 1);
        clear_current_request();
        return Ok(());
    }

    // v0.87.0: Try Brain fast path for simple questions (RAM, CPU, disk, health)
    // This bypasses LLM entirely for known patterns
    if let Some(fast_answer) = try_fast_answer(question) {
        // Show the question
        spinner::print_question(question);
        println!();

        // Print the answer in the new format
        print_final_answer(&fast_answer.text, fast_answer.reliability, &fast_answer.origin, fast_answer.duration_ms);

        // Update XP store
        let mut xp_store = XpStore::load();
        let xp_line = xp_store.anna_self_solve(question);
        if streaming_debug::is_debug_enabled() {
            println!("{}", xp_line);
        }

        // Log completion
        logging::with_current_request(|ctx| {
            ctx.set_result(fast_answer.reliability, RequestStatus::Ok);
            log_request(&ctx.to_trace());
        });
        clear_current_request();

        return Ok(());
    }

    // v0.43.0: Check if debug streaming is enabled
    if streaming_debug::is_debug_enabled() {
        return run_ask_with_debug_stream(question, &request_id).await;
    }

    // v0.15.8: Show user question with old-school style
    spinner::print_question(question);

    let daemon = client::DaemonClient::new();

    // Check daemon health
    if !daemon.is_healthy().await {
        // Log daemon unavailable
        logging::logger().write_daemon(
            &LogEntry::new(
                LogLevel::Error,
                LogComponent::Request,
                "Daemon not available",
            )
            .with_request_id(&request_id),
        );

        // v0.6.0: ASCII-only error output
        eprintln!("[ERROR] Anna daemon is not running");
        eprintln!("   Run: {} to start", "sudo systemctl start annad".cyan());

        // Log and clear request context
        logging::with_current_request(|ctx| {
            ctx.set_result(0.0, RequestStatus::Failed);
            log_request(&ctx.to_trace());
        });
        clear_current_request();

        std::process::exit(1);
    }

    // v0.15.8: Start spinner while thinking
    let thinking = spinner::Spinner::new("thinking...");

    // v0.10.0: Use evidence-based answer engine
    match daemon.answer(question).await {
        Ok(final_answer) => {
            // v0.10.0: Log request completion
            logging::with_current_request(|ctx| {
                let status = if final_answer.is_refusal {
                    RequestStatus::Failed
                } else if final_answer.scores.overall >= 0.9 {
                    RequestStatus::Ok
                } else if final_answer.scores.overall >= 0.7 {
                    RequestStatus::Degraded
                } else {
                    RequestStatus::Failed
                };
                ctx.set_result(final_answer.scores.overall, status);
                log_request(&ctx.to_trace());
            });

            // Log completion
            logging::logger().write_daemon(
                &LogEntry::new(LogLevel::Info, LogComponent::Request, "Request completed")
                    .with_request_id(&request_id)
                    .with_fields(serde_json::json!({
                        "confidence": final_answer.scores.overall,
                        "is_refusal": final_answer.is_refusal,
                        "citations_count": final_answer.citations.len(),
                        "loop_iterations": final_answer.loop_iterations
                    })),
            );

            // v0.88.0: Process XP events for Junior/Senior (wires to XpLog for 24h metrics)
            process_llm_xp_events(question, &final_answer);

            clear_current_request();

            // v0.15.8: Stop spinner and show timing
            let elapsed = thinking.finish();

            // v0.81.0: Check for QA mode - output JSON instead of TUI
            if std::env::var("ANNA_QA_MODE").is_ok() {
                // QA mode: output machine-readable JSON with timing and dialog trace
                let qa_output = final_answer.to_qa_output();
                println!("{}", serde_json::to_string_pretty(&qa_output).unwrap_or_default());
            } else {
                // Normal mode: Display structured answer with headline/details/evidence/reliability
                output::display_structured_answer(&final_answer, elapsed);
            }
            Ok(())
        }
        Err(e) => {
            // v0.15.8: Stop spinner on error
            thinking.stop();

            // Log error
            logging::logger().write_daemon(
                &LogEntry::new(
                    LogLevel::Error,
                    LogComponent::Request,
                    "Answer request failed",
                )
                .with_request_id(&request_id)
                .with_fields(serde_json::json!({
                    "error": e.to_string()
                })),
            );

            logging::with_current_request(|ctx| {
                ctx.set_result(0.0, RequestStatus::Failed);
                log_request(&ctx.to_trace());
            });
            clear_current_request();

            // Display error
            output::display_error(&format!("Failed to get answer: {}", e));
            Err(e)
        }
    }
}

/// v0.43.0: Ask with live debug streaming
/// Uses the streaming endpoint to display real-time debug events
async fn run_ask_with_debug_stream(question: &str, request_id: &str) -> Result<()> {
    let daemon = client::DaemonClient::new();

    // Check daemon health first
    if !daemon.is_healthy().await {
        // Log daemon unavailable
        logging::logger().write_daemon(
            &LogEntry::new(
                LogLevel::Error,
                LogComponent::Request,
                "Daemon not available",
            )
            .with_request_id(request_id),
        );

        // v0.6.0: ASCII-only error output
        eprintln!("[ERROR] Anna daemon is not running");
        eprintln!("   Run: {} to start", "sudo systemctl start annad".cyan());

        // Log and clear request context
        logging::with_current_request(|ctx| {
            ctx.set_result(0.0, RequestStatus::Failed);
            log_request(&ctx.to_trace());
        });
        clear_current_request();

        std::process::exit(1);
    }

    // Stream debug events
    match streaming_debug::stream_answer_with_debug(question).await {
        Ok(_) => {
            // v0.43.0: Log request completion
            logging::with_current_request(|ctx| {
                ctx.set_result(0.9, RequestStatus::Ok);
                log_request(&ctx.to_trace());
            });
            clear_current_request();
            Ok(())
        }
        Err(e) => {
            // Log error
            logging::logger().write_daemon(
                &LogEntry::new(
                    LogLevel::Error,
                    LogComponent::Request,
                    "Debug stream failed",
                )
                .with_request_id(request_id)
                .with_fields(serde_json::json!({
                    "error": e.to_string()
                })),
            );

            logging::with_current_request(|ctx| {
                ctx.set_result(0.0, RequestStatus::Failed);
                log_request(&ctx.to_trace());
            });
            clear_current_request();

            // Display error
            output::display_error(&format!("Debug stream failed: {}", e));
            Err(e)
        }
    }
}

/// v0.15.0: Instant version display - no LLM, shows all components
async fn run_version_instant() -> Result<()> {
    use anna_common::THIN_SEPARATOR;

    let daemon = client::DaemonClient::new();

    println!();
    println!("{}", "ANNA VERSION".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // annactl version (always available)
    println!("  {}  annactl v{}", "*".cyan(), env!("CARGO_PKG_VERSION"));

    // annad version (requires daemon)
    if daemon.is_healthy().await {
        match daemon.health().await {
            Ok(health) => {
                // v0.16.5: Human-readable uptime
                let uptime_str = format_uptime_human(health.uptime_seconds);
                println!(
                    "  {}  annad v{} (uptime {})",
                    "+".bright_green(),
                    health.version,
                    uptime_str.cyan()
                );
            }
            Err(_) => {
                println!("  {}  annad (version unavailable)", "~".yellow());
            }
        }
    } else {
        println!("  {}  annad not running", "!".bright_red());
    }

    // Ollama version
    match check_ollama_version() {
        Some(ver) => {
            println!("  {}  ollama {}", "+".bright_green(), ver);
        }
        None => {
            println!("  {}  ollama not available", "!".bright_red());
        }
    }

    println!("{}", THIN_SEPARATOR);
    println!();

    Ok(())
}

/// Check Ollama version
fn check_ollama_version() -> Option<String> {
    use std::process::Command;

    let output = Command::new("ollama").args(["--version"]).output().ok()?;

    if output.status.success() {
        let ver = String::from_utf8_lossy(&output.stdout);
        // Parse "ollama version is 0.5.4" -> "v0.5.4"
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

/// Help via LLM pipeline - Anna explains how to use herself
async fn run_help_via_llm() -> Result<()> {
    let daemon = client::DaemonClient::new();

    let help_question =
        "How do I use Anna? Show usage, natural language configuration, and examples.";

    // Load v0.5.0 config
    let config = AnnaConfigV5::load();

    // Process through orchestrator for consistent formatting
    let result = orchestrator::process_internal_query(
        help_question,
        &daemon,
        orchestrator::InternalQueryType::Help { config },
    )
    .await?;

    output::display_response(&result);
    Ok(())
}

/// v0.9.0: Status command - compact summary of daemon, LLM, probes, update state
async fn run_status() -> Result<()> {
    use anna_common::THIN_SEPARATOR;

    let daemon = client::DaemonClient::new();
    let config = AnnaConfigV5::load();

    println!();
    println!("{}", "ANNA STATUS".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // Version info
    println!("  {}  annactl v{}", "*".cyan(), env!("CARGO_PKG_VERSION"));

    // Daemon status
    if daemon.is_healthy().await {
        match daemon.health().await {
            Ok(health) => {
                // v0.16.5: Human-readable uptime
                let uptime_str = format_uptime_human(health.uptime_seconds);
                println!(
                    "  {}  annad v{} (uptime: {})",
                    "+".bright_green(),
                    health.version,
                    uptime_str.cyan()
                );

                // v0.16.5: List probe names instead of just count
                if health.probe_names.is_empty() {
                    println!("  {}  No probes available", "!".bright_red());
                } else {
                    println!(
                        "  {}  {} probes:",
                        "+".bright_green(),
                        health.probes_available
                    );
                    for probe in &health.probe_names {
                        println!("       •  {}", probe.dimmed());
                    }
                }
            }
            Err(_) => {
                println!("  {}  annad running (details unavailable)", "~".yellow());
            }
        }
    } else {
        println!("  {}  annad not running", "!".bright_red());
    }

    // LLM status
    let llm_status = check_llm_status();
    match llm_status.as_str() {
        "running" => println!("  {}  Ollama: running", "+".bright_green()),
        "stopped" => println!("  {}  Ollama: stopped", "!".bright_red()),
        other => println!("  {}  Ollama: {}", "~".yellow(), other),
    }

    // Model info - show role-specific models if configured
    if config.llm.needs_role_model_migration() {
        println!(
            "  {}  Model: {} ({})",
            "*".cyan(),
            config.llm.preferred_model,
            config.llm.selection_mode.as_str()
        );
        println!(
            "  {}  {} (run installer to optimize)",
            "~".yellow(),
            "Legacy single-model config".yellow()
        );
    } else {
        println!(
            "  {}  Junior: {} (fast)",
            "*".cyan(),
            config.llm.get_junior_model()
        );
        println!(
            "  {}  Senior: {} (smart)",
            "*".cyan(),
            config.llm.get_senior_model()
        );
    }

    println!("{}", THIN_SEPARATOR);

    // v0.40.1: RPG Progression section
    // v0.71.0: Now async to fetch stats from daemon API
    display_progression_section(&daemon).await;

    // v0.86.0: LLM Agents section
    display_llm_agents_section();

    // v0.89.0: Debug Mode section (only shown when enabled)
    if debug_is_enabled() {
        println!("{}", "DEBUG MODE".bright_white().bold());
        println!("{}", THIN_SEPARATOR);
        println!("  {}  Live debug stream: {}", "*".cyan(), "ENABLED".bright_green());
        println!("{}", THIN_SEPARATOR);
        println!();
    }

    // Update state
    println!("{}", "UPDATE STATE".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // Get update state from daemon if available
    if daemon.is_healthy().await {
        match daemon.update_state().await {
            Ok(state) => {
                println!("  {}  Current: v{}", "*".cyan(), env!("CARGO_PKG_VERSION"));
                if let Some(latest) = &state.latest_version {
                    // v0.30.2: Use proper semantic version comparison
                    // Only show "Available" if the latest version is actually newer
                    if is_version_newer(latest, env!("CARGO_PKG_VERSION")) {
                        println!(
                            "  {}  Available: v{} ({})",
                            "^".bright_green(),
                            latest,
                            state.status
                        );
                    } else {
                        println!("  {}  Up to date", "+".bright_green());
                    }
                } else {
                    println!("  {}  Latest version: unknown", "?".dimmed());
                }

                // Show update mode
                if config.update.enabled {
                    println!(
                        "  {}  Auto-update: {} (every {})",
                        "*".cyan(),
                        if config.core.mode == anna_common::CoreMode::Dev {
                            "dev mode"
                        } else {
                            "enabled"
                        },
                        format_uptime_human(config.update.effective_interval())
                    );
                } else {
                    println!("  {}  Auto-update: disabled", "-".dimmed());
                }

                // Show last check time with human-readable "ago" format
                if let Some(last_check) = &state.last_check {
                    let ago = format_time_ago(last_check);
                    println!("  {}  Last check: {}", "*".cyan(), ago.dimmed());
                }
            }
            Err(_) => {
                println!("  {}  Update state unavailable", "?".dimmed());
            }
        }
    } else {
        println!(
            "  {}  Daemon not running, update state unavailable",
            "!".bright_red()
        );
    }

    println!("{}", THIN_SEPARATOR);

    // v0.16.5: Detailed self-health with individual component status
    let health_report = self_health::run_all_probes();
    println!("{}", "SELF-HEALTH".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // Overall status header
    match health_report.overall {
        OverallHealth::Healthy => {
            println!(
                "  {}  Overall: {} ({} components)",
                "+".bright_green(),
                "healthy".bright_green(),
                health_report.components.len()
            );
        }
        OverallHealth::Degraded => {
            println!("  {}  Overall: {}", "~".yellow(), "degraded".yellow());
        }
        OverallHealth::Critical => {
            println!(
                "  {}  Overall: {}",
                "!".bright_red(),
                "CRITICAL".bright_red().bold()
            );
        }
        OverallHealth::Unknown => {
            println!("  {}  Overall: {}", "?".dimmed(), "unknown".dimmed());
        }
    }

    // List each component with its status
    for component in &health_report.components {
        let (icon, status_str) = match component.status {
            anna_common::ComponentStatus::Healthy => (
                "+".bright_green().to_string(),
                "ok".bright_green().to_string(),
            ),
            anna_common::ComponentStatus::Degraded => {
                ("~".yellow().to_string(), "degraded".yellow().to_string())
            }
            anna_common::ComponentStatus::Critical => (
                "!".bright_red().to_string(),
                "CRITICAL".bright_red().to_string(),
            ),
            anna_common::ComponentStatus::Unknown => {
                ("?".dimmed().to_string(), "unknown".dimmed().to_string())
            }
        };

        // Show component with its status and message if not healthy
        if component.status.is_healthy() {
            println!("       {}  {}: {}", icon, component.name, status_str);
        } else {
            println!(
                "       {}  {}: {} - {}",
                icon,
                component.name,
                status_str,
                component.message.dimmed()
            );
        }
    }
    println!("{}", THIN_SEPARATOR);
    println!();

    Ok(())
}

/// v0.15.9: Format timestamp as human-readable "X ago"
fn format_time_ago(rfc3339: &str) -> String {
    use chrono::{DateTime, Utc};

    // Parse RFC 3339 timestamp
    let parsed: Result<DateTime<Utc>, _> = rfc3339.parse();

    match parsed {
        Ok(timestamp) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(timestamp);
            let secs = duration.num_seconds();

            if secs < 0 {
                // Future time - shouldn't happen
                rfc3339.to_string()
            } else if secs < 60 {
                format!("{}s ago", secs)
            } else if secs < 3600 {
                let mins = secs / 60;
                format!("{}m ago", mins)
            } else if secs < 86400 {
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                if mins > 0 {
                    format!("{}h {}m ago", hours, mins)
                } else {
                    format!("{}h ago", hours)
                }
            } else {
                let days = secs / 86400;
                format!("{}d ago", days)
            }
        }
        Err(_) => rfc3339.to_string(), // Fallback to raw timestamp
    }
}

/// v0.16.5: Format uptime seconds as human-readable duration
/// Examples: "45s", "3m 12s", "2h 15m", "5d 3h 20m", "2w 1d"
fn format_uptime_human(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;

    if seconds < MINUTE {
        format!("{}s", seconds)
    } else if seconds < HOUR {
        let mins = seconds / MINUTE;
        let secs = seconds % MINUTE;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else if seconds < DAY {
        let hours = seconds / HOUR;
        let mins = (seconds % HOUR) / MINUTE;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else if seconds < WEEK {
        let days = seconds / DAY;
        let hours = (seconds % DAY) / HOUR;
        let mins = (seconds % HOUR) / MINUTE;
        if hours > 0 {
            format!("{}d {}h {}m", days, hours, mins)
        } else {
            format!("{}d", days)
        }
    } else {
        let weeks = seconds / WEEK;
        let days = (seconds % WEEK) / DAY;
        let hours = (seconds % DAY) / HOUR;
        if days > 0 {
            format!("{}w {}d {}h", weeks, days, hours)
        } else {
            format!("{}w", weeks)
        }
    }
}

/// Check if Ollama is running
fn check_llm_status() -> String {
    use std::process::Command;

    // Try systemctl first
    if let Ok(output) = Command::new("systemctl")
        .args(["is-active", "ollama"])
        .output()
    {
        let status = String::from_utf8_lossy(&output.stdout);
        if status.trim() == "active" {
            return "running".to_string();
        }
    }

    // Try pgrep
    if let Ok(output) = Command::new("pgrep").arg("ollama").output() {
        if output.status.success() {
            return "running".to_string();
        }
    }

    "stopped".to_string()
}

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// v0.7.0: Run self-health check on REPL startup
fn run_startup_health_check() {
    // Run probes with auto-repair
    let report = self_health::run_with_auto_repair();

    // Display notifications based on health status
    match report.overall {
        OverallHealth::Healthy => {
            // Silent if healthy - no notification needed
        }
        OverallHealth::Degraded => {
            println!(
                "{}  Self-health: {}",
                "[NOTE]".yellow(),
                "degraded".yellow()
            );
            // Show degraded components
            for component in &report.components {
                if !component.status.is_healthy() {
                    println!("   * {}: {}", component.name.yellow(), component.message);
                }
            }
            println!();
        }
        OverallHealth::Critical => {
            println!(
                "{}  Self-health: {}",
                "[WARNING]".bright_red(),
                "critical".bright_red()
            );
            // Show critical components
            for component in &report.components {
                if !component.status.is_healthy() {
                    println!(
                        "   * {}: {}",
                        component.name.bright_red(),
                        component.message
                    );
                }
            }
            println!();
        }
        OverallHealth::Unknown => {
            println!("{}  Self-health: {}", "[NOTE]".dimmed(), "unknown".dimmed());
            println!();
        }
    }

    // Show auto-repairs that were executed
    if !report.repairs_executed.is_empty() {
        println!("{}  Auto-repairs executed:", "[AUTO-REPAIR]".bright_green());
        for repair in &report.repairs_executed {
            let status = if repair.success {
                "+".bright_green().to_string()
            } else {
                "!".bright_red().to_string()
            };
            println!("   {} {}", status, repair.message);
        }
        println!();
    }

    // Show manual actions required
    let manual_actions: Vec<_> = report
        .repairs_available
        .iter()
        .filter(|r| r.safety == RepairSafety::WarnOnly)
        .collect();

    if !manual_actions.is_empty() {
        println!("{}  Manual action required:", "[ACTION]".yellow());
        for repair in manual_actions {
            println!(
                "   * {}: {}",
                repair.description.yellow(),
                repair.command.cyan()
            );
        }
        println!();
    }
}

/// v0.40.1: Display RPG progression section in status output
/// v0.71.0: Now async, fetches stats from daemon API to solve permission issues
async fn display_progression_section(daemon: &client::DaemonClient) {
    use anna_common::THIN_SEPARATOR;

    println!("{}", "ANNA PROGRESSION".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // v0.71.0: Try to fetch stats from daemon first (authoritative source)
    // Falls back to local file if daemon unavailable
    let snapshot = if daemon.is_healthy().await {
        match daemon.stats().await {
            Ok(stats) => stats,
            Err(_) => {
                // Fallback to local file
                StatsEngine::load_default().unwrap_or_default().snapshot()
            }
        }
    } else {
        // Daemon not running - use local file
        StatsEngine::load_default().unwrap_or_default().snapshot()
    };

    // Level and Title
    let level = snapshot.progression.level.value();
    let title = snapshot.progression.title.to_string();
    let total_xp = snapshot.progression.total_xp;

    // Color title based on level bands
    let title_colored = if level < 5 {
        title.dimmed().to_string()
    } else if level < 15 {
        title.cyan().to_string()
    } else if level < 30 {
        title.bright_cyan().to_string()
    } else if level < 50 {
        title.bright_green().to_string()
    } else if level < 70 {
        title.bright_yellow().to_string()
    } else if level < 90 {
        title.bright_magenta().to_string()
    } else {
        title.bright_red().bold().to_string()
    };

    println!(
        "  {}  Level {} - {}",
        "*".cyan(),
        format!("{}", level).bright_white().bold(),
        title_colored
    );

    // XP Progress bar
    let progress_pct = snapshot.progression.progress_percent() as usize;
    let xp_to_next = snapshot.progression.xp_to_next_level();

    // Build progress bar (20 chars wide)
    let bar_width = 20;
    let filled = (progress_pct * bar_width) / 100;
    let empty = bar_width - filled;
    let bar = format!(
        "[{}{}]",
        "=".repeat(filled).bright_green(),
        "-".repeat(empty).dimmed()
    );

    if level < 99 {
        println!(
            "  {}  {} {}% ({} XP to next)",
            "*".cyan(),
            bar,
            progress_pct,
            xp_to_next
        );
    } else {
        println!(
            "  {}  {} MAX LEVEL",
            "+".bright_green(),
            bar.bright_yellow()
        );
    }

    // Total XP
    println!("  {}  Total XP: {}", "*".cyan(), format_xp(total_xp));

    println!("{}", THIN_SEPARATOR);

    // Statistics section
    println!("{}", "PERFORMANCE STATS".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    let global = &snapshot.global;

    if global.total_questions == 0 {
        println!("  {}  No questions answered yet", "?".dimmed());
    } else {
        // Questions answered
        println!(
            "  {}  Questions: {} ({} successful)",
            "*".cyan(),
            global.total_questions,
            global.total_successful
        );

        // Success rate with color coding
        let success_rate = global.success_rate();
        let rate_str = format!("{:.1}%", success_rate);
        let rate_colored = if success_rate >= 90.0 {
            rate_str.bright_green().to_string()
        } else if success_rate >= 70.0 {
            rate_str.bright_cyan().to_string()
        } else if success_rate >= 50.0 {
            rate_str.yellow().to_string()
        } else {
            rate_str.bright_red().to_string()
        };
        println!("  {}  Success rate: {}", "*".cyan(), rate_colored);

        // Average reliability (v0.72.0: show as percentage)
        let reliability_pct = (global.avg_reliability * 100.0).round() as u32;
        let reliability_str = format!("{}%", reliability_pct);
        let rel_colored = if reliability_pct >= 90 {
            reliability_str.bright_green().to_string()
        } else if reliability_pct >= 70 {
            reliability_str.bright_cyan().to_string()
        } else {
            reliability_str.yellow().to_string()
        };
        println!("  {}  Avg reliability: {}", "*".cyan(), rel_colored);

        // Average latency (v0.72.0: human-friendly format)
        let latency_str = if global.avg_latency_ms < 1000.0 {
            format!("{}ms", global.avg_latency_ms.round() as u64)
        } else {
            format!("{:.1}s", global.avg_latency_ms / 1000.0)
        };
        println!(
            "  {}  Avg latency: {}",
            "*".cyan(),
            latency_str
        );

        // Patterns tracked
        println!(
            "  {}  Patterns tracked: {} ({} improved)",
            "*".cyan(),
            global.distinct_patterns,
            global.patterns_improved
        );
    }

    println!("{}", THIN_SEPARATOR);

    // v0.84.0: 24-hour XP metrics
    println!("{}", "24-HOUR XP METRICS".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    let xp_log = XpLog::new();
    let metrics = xp_log.metrics_24h();

    if metrics.total_events == 0 {
        println!("  {}  No XP events in the last 24 hours", "?".dimmed());
    } else {
        // Net XP with color
        let net_str = if metrics.net_xp >= 0 {
            format!("+{}", metrics.net_xp).bright_green().to_string()
        } else {
            format!("{}", metrics.net_xp).bright_red().to_string()
        };

        println!("  {}  Net XP (24h): {}", "*".cyan(), net_str);
        println!(
            "  {}  Gained: +{} | Lost: -{}",
            "*".cyan(),
            format!("{}", metrics.xp_gained).bright_green(),
            format!("{}", metrics.xp_lost).bright_red()
        );
        println!(
            "  {}  Events: {} ({} positive, {} negative)",
            "*".cyan(),
            metrics.total_events,
            metrics.positive_events,
            metrics.negative_events
        );
    }

    println!("{}", THIN_SEPARATOR);
}

/// Format XP with comma separators for readability
fn format_xp(xp: u64) -> String {
    if xp < 1000 {
        format!("{}", xp)
    } else if xp < 1_000_000 {
        format!("{},{:03}", xp / 1000, xp % 1000)
    } else {
        format!(
            "{},{:03},{:03}",
            xp / 1_000_000,
            (xp % 1_000_000) / 1000,
            xp % 1000
        )
    }
}

// ============================================================================
// v0.87.0: Print Final Answer - Always visible answer block
// ============================================================================

/// Print the final answer in the standardized format
/// This ensures every question gets a clear, visible answer
fn print_final_answer(text: &str, reliability: f64, origin: &str, duration_ms: u64) {
    use anna_common::THIN_SEPARATOR;

    println!();
    println!("{}", THIN_SEPARATOR);
    println!("{}", "Anna".bright_white().bold());
    println!("{}", THIN_SEPARATOR);
    println!();
    println!("{}", text);
    println!();
    println!("{}", THIN_SEPARATOR);

    // Reliability with color
    let rel_pct = format!("{:.0}%", reliability * 100.0);
    let rel_label = if reliability >= 0.9 {
        format!("{} ({})", rel_pct.bright_green(), "Green".bright_green())
    } else if reliability >= 0.7 {
        format!("{} ({})", rel_pct.yellow(), "Yellow".yellow())
    } else {
        format!("{} ({})", rel_pct.bright_red(), "Red".bright_red())
    };
    println!("Reliability: {}", rel_label);

    // Origin
    println!("Origin: {}", origin.cyan());

    // Duration
    let dur_str = if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else {
        format!("{:.2}s", duration_ms as f64 / 1000.0)
    };
    println!("Duration: {}", dur_str);

    println!("{}", THIN_SEPARATOR);
    println!();
}

// ============================================================================
// v0.91.0: Debug Intent Handling (Brain Fast Path)
// ============================================================================

/// Handle debug mode control commands directly in Brain layer (no LLM)
/// Returns a human-readable response for the debug intent
fn handle_debug_intent(_question: &str, intent: DebugIntent) -> String {
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
        DebugIntent::None => {
            // This shouldn't happen - is_debug_intent() check guards against it
            "Not a debug command.".to_string()
        }
    }
}

// ============================================================================
// v0.86.0: Display LLM Agents Section for Status
// ============================================================================

/// Display the LLM agents XP section in status
fn display_llm_agents_section() {
    use anna_common::THIN_SEPARATOR;

    let xp_store = XpStore::load();
    let policy = DecisionPolicy::load();

    println!();
    println!("{}", "LLM AGENTS".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // Junior with circuit breaker status
    let junior_trust = format!("trust {}", xp_store.junior.trust_pct());
    let junior_trust_color = if xp_store.junior.is_high_trust() {
        junior_trust.bright_green().to_string()
    } else if xp_store.junior.is_low_trust() {
        junior_trust.bright_red().to_string()
    } else {
        junior_trust.yellow().to_string()
    };

    let junior_health = if policy.junior_health.is_degraded {
        "DEGRADED".bright_red().to_string()
    } else {
        "OK".bright_green().to_string()
    };

    println!(
        "  {}  Junior: Level {} - {} ({}) [{}]",
        "*".cyan(),
        xp_store.junior.level,
        xp_store.junior.title(),
        junior_trust_color,
        junior_health
    );
    println!(
        "       Good plans: {}   Bad plans: {}   Timeouts: {}",
        xp_store.junior_stats.good_plans.to_string().bright_green(),
        xp_store.junior_stats.bad_plans.to_string().bright_red(),
        xp_store.junior_stats.timeouts.to_string().yellow()
    );
    if policy.path_metrics.junior_calls > 0 {
        println!(
            "       Avg latency: {}ms ({} calls)",
            policy.path_metrics.junior_latency_avg_ms.to_string().dimmed(),
            policy.path_metrics.junior_calls.to_string().dimmed()
        );
    }

    // Senior with circuit breaker status
    let senior_trust = format!("trust {}", xp_store.senior.trust_pct());
    let senior_trust_color = if xp_store.senior.is_high_trust() {
        senior_trust.bright_green().to_string()
    } else if xp_store.senior.is_low_trust() {
        senior_trust.bright_red().to_string()
    } else {
        senior_trust.yellow().to_string()
    };

    let senior_health = if policy.senior_health.is_degraded {
        "DEGRADED".bright_red().to_string()
    } else {
        "OK".bright_green().to_string()
    };

    println!(
        "  {}  Senior: Level {} - {} ({}) [{}]",
        "*".cyan(),
        xp_store.senior.level,
        xp_store.senior.title(),
        senior_trust_color,
        senior_health
    );
    println!(
        "       Approvals: {}    Fix&Accept: {}  Rubber-stamps blocked: {}",
        xp_store.senior_stats.approvals.to_string().bright_green(),
        xp_store.senior_stats.fix_and_accept.to_string().yellow(),
        xp_store.senior_stats.rubber_stamps_blocked.to_string().bright_red()
    );
    if policy.path_metrics.senior_calls > 0 {
        println!(
            "       Avg latency: {}ms ({} calls)",
            policy.path_metrics.senior_latency_avg_ms.to_string().dimmed(),
            policy.path_metrics.senior_calls.to_string().dimmed()
        );
    }

    println!("{}", THIN_SEPARATOR);

    // v0.92.0: Path Metrics summary
    if policy.path_metrics.brain_calls > 0 || policy.path_metrics.full_calls > 0 {
        println!();
        println!("{}", "PATH METRICS".bright_white().bold());
        println!("{}", THIN_SEPARATOR);

        if policy.path_metrics.brain_calls > 0 {
            println!(
                "  {}  Brain: {}ms avg ({} calls)",
                "+".bright_green(),
                policy.path_metrics.brain_latency_avg_ms,
                policy.path_metrics.brain_calls
            );
        }
        if policy.path_metrics.full_calls > 0 {
            println!(
                "  {}  Full orchestration: {}ms avg ({} calls)",
                "*".cyan(),
                policy.path_metrics.full_latency_avg_ms,
                policy.path_metrics.full_calls
            );
        }
        println!("{}", THIN_SEPARATOR);
    }

    // Low trust warnings
    if let Some(warning) = xp_store.low_trust_warning() {
        println!();
        println!("{}", "[!] Trust Warning".bright_red().bold());
        for line in warning.lines() {
            println!("    {}", line.dimmed());
        }
    }

    // Circuit breaker warnings
    if policy.junior_health.is_degraded || policy.senior_health.is_degraded {
        println!();
        println!("{}", "[!] Circuit Breaker Warning".bright_red().bold());
        if policy.junior_health.is_degraded {
            println!(
                "    {} Junior is in degraded state (timeouts={}, failures={})",
                "!".bright_red(),
                policy.junior_health.recent_timeouts,
                policy.junior_health.recent_failures
            );
        }
        if policy.senior_health.is_degraded {
            println!(
                "    {} Senior is in degraded state (timeouts={}, failures={})",
                "!".bright_red(),
                policy.senior_health.recent_timeouts,
                policy.senior_health.recent_failures
            );
        }
    }
}

// ============================================================================
// v0.88.0: XP Event Processing for Junior/Senior
// ============================================================================

/// Process XP events from a FinalAnswer and log them
/// This wires up the Junior and Senior XP events that were previously missing
fn process_llm_xp_events(question: &str, answer: &FinalAnswer) {
    let xp_log = XpLog::new();
    let mut xp_store = XpStore::load();

    // Determine Junior event based on senior verdict
    let senior_verdict = answer.senior_verdict.as_deref().unwrap_or("unknown");
    let junior_had_draft = answer.junior_had_draft;
    let confidence = answer.scores.overall;

    // Junior XP events
    if junior_had_draft && (senior_verdict == "approve" || senior_verdict == "fix_and_accept") {
        // Junior provided a draft that was accepted
        let event = XpEvent::new(XpEventType::JuniorCleanProposal, question)
            .with_context(&format!("verdict={}", senior_verdict));
        let _ = xp_log.append(&event);
        let _ = xp_store.junior_plan_good("");

        if streaming_debug::is_debug_enabled() {
            println!("{}", event.format_log());
        }
    } else if !junior_had_draft || senior_verdict == "refuse" {
        // Junior failed to provide a usable draft
        let event = XpEvent::new(XpEventType::JuniorBadCommand, question)
            .with_context(&format!("verdict={}", senior_verdict));
        let _ = xp_log.append(&event);
        let _ = xp_store.junior_plan_bad();

        if streaming_debug::is_debug_enabled() {
            println!("{}", event.format_log());
        }
    }

    // Senior XP events based on verdict and confidence
    match senior_verdict {
        "approve" if confidence >= 0.9 => {
            // Green approval
            let event = XpEvent::new(XpEventType::SeniorGreenApproval, question)
                .with_context(&format!("score={:.0}%", confidence * 100.0));
            let _ = xp_log.append(&event);
            let _ = xp_store.senior_approve_correct(confidence);

            if streaming_debug::is_debug_enabled() {
                println!("{}", event.format_log());
            }
        }
        "fix_and_accept" => {
            // Senior fixed Junior's work
            let _ = xp_store.senior_fix_accept_good();
            let _ = xp_store.junior_needs_fix();

            if streaming_debug::is_debug_enabled() {
                println!("[XP] Senior: +4 XP (fix_and_accept_good)");
            }
        }
        "refuse" => {
            let _ = xp_store.senior_refusal();

            if streaming_debug::is_debug_enabled() {
                println!("[XP] Senior: refusal recorded");
            }
        }
        _ => {
            // Unknown verdict or approve with lower confidence
            if confidence >= 0.7 {
                let _ = xp_store.senior_approve_correct(confidence);
            }
        }
    }

    // Handle timeout/failure cases
    if let Some(ref failure_cause) = answer.failure_cause {
        match failure_cause.as_str() {
            "timeout_or_latency" => {
                let event = XpEvent::new(XpEventType::LlmTimeoutFallback, question);
                let _ = xp_log.append(&event);
                let _ = xp_store.anna_timeout();

                if streaming_debug::is_debug_enabled() {
                    println!("{}", event.format_log());
                }
            }
            "unsupported_domain" => {
                let event = XpEvent::new(XpEventType::LowReliabilityRefusal, question);
                let _ = xp_log.append(&event);

                if streaming_debug::is_debug_enabled() {
                    println!("{}", event.format_log());
                }
            }
            _ => {}
        }
    }
}
