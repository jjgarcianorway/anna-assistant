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
//!   - annactl -V | --version | version  Show version info
//!   - annactl -h | --help | help        Show help info

// Allow dead code for features planned but not yet fully wired
#![allow(dead_code)]
#![allow(unused_imports)]

mod client;
mod llm_client;
mod orchestrator;
mod output;

use anna_common::{
    clear_current_request, generate_request_id, init_logger, log_request, logging, self_health,
    set_current_request, AnnaConfigV5, HardwareProfile, LogComponent, LogEntry, LogLevel,
    OverallHealth, RepairSafety, RequestContext, RequestStatus,
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

        // Simple version flag - fast, no daemon contact
        [flag] if flag == "-V" || flag == "--version" => {
            println!("annactl {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        // Detailed version via daemon
        [flag]
            if flag.eq_ignore_ascii_case("-v")
                || flag.eq_ignore_ascii_case("version") =>
        {
            run_version_via_llm().await
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
        if matches!(
            input.to_lowercase().as_str(),
            "version" | "-v" | "--version"
        ) {
            run_version_via_llm().await?;
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

            clear_current_request();

            // v0.10.0: Display evidence-based answer
            output::display_final_answer(&final_answer);
            Ok(())
        }
        Err(e) => {
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

/// Version via LLM pipeline - Anna answers about herself
async fn run_version_via_llm() -> Result<()> {
    let daemon = client::DaemonClient::new();

    // Build internal question for version info
    let version_question =
        "What is your version? Report: mode, update status, LLM config, daemon status.";

    // Check if daemon is healthy and get status
    let daemon_status = if daemon.is_healthy().await {
        match daemon.health().await {
            Ok(health) => format!(
                "running (v{}, uptime: {}s, {} probes)",
                health.version, health.uptime_seconds, health.probes_available
            ),
            Err(_) => "running (details unavailable)".to_string(),
        }
    } else {
        "stopped".to_string()
    };

    // Get probe count
    let probe_count = if daemon.is_healthy().await {
        daemon
            .list_probes()
            .await
            .map(|p| p.probes.len())
            .unwrap_or(0)
    } else {
        0
    };

    // Load v0.5.0 config and detect hardware
    let config = AnnaConfigV5::load();
    let hardware = HardwareProfile::detect();

    // Process through orchestrator for consistent formatting
    let result = orchestrator::process_internal_query(
        version_question,
        &daemon,
        orchestrator::InternalQueryType::Version {
            version: env!("CARGO_PKG_VERSION").to_string(),
            daemon_status,
            probe_count,
            config,
            hardware,
        },
    )
    .await?;

    output::display_response(&result);
    Ok(())
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
                println!(
                    "  {}  annad v{} (uptime: {}s)",
                    "+".bright_green(),
                    health.version,
                    health.uptime_seconds
                );
                println!(
                    "  {}  {} probes available",
                    "+".bright_green(),
                    health.probes_available
                );
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

    // Model info
    println!(
        "  {}  Model: {} ({})",
        "*".cyan(),
        config.llm.preferred_model,
        config.llm.selection_mode.as_str()
    );

    println!("{}", THIN_SEPARATOR);

    // Update state
    println!("{}", "UPDATE STATE".bright_white().bold());
    println!("{}", THIN_SEPARATOR);

    // Get update state from daemon if available
    if daemon.is_healthy().await {
        match daemon.update_state().await {
            Ok(state) => {
                println!("  {}  Current: v{}", "*".cyan(), env!("CARGO_PKG_VERSION"));
                if let Some(latest) = &state.latest_version {
                    if latest != env!("CARGO_PKG_VERSION") {
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
                        "  {}  Auto-update: {} (every {}s)",
                        "*".cyan(),
                        if config.core.mode == anna_common::CoreMode::Dev {
                            "dev mode"
                        } else {
                            "enabled"
                        },
                        config.update.effective_interval()
                    );
                } else {
                    println!("  {}  Auto-update: disabled", "-".dimmed());
                }

                // Show last check time
                if let Some(last_check) = &state.last_check {
                    println!("  {}  Last check: {}", "*".cyan(), last_check);
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

    // Self-health summary (compact)
    let health_report = self_health::run_all_probes();
    println!("{}", "SELF-HEALTH".bright_white().bold());
    println!("{}", THIN_SEPARATOR);
    match health_report.overall {
        OverallHealth::Healthy => {
            println!(
                "  {}  All components healthy ({} checked)",
                "+".bright_green(),
                health_report.components.len()
            );
        }
        OverallHealth::Degraded => {
            let degraded: Vec<_> = health_report
                .components
                .iter()
                .filter(|c| !c.status.is_healthy())
                .collect();
            println!(
                "  {}  Degraded: {}",
                "~".yellow(),
                degraded
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        OverallHealth::Critical => {
            let critical: Vec<_> = health_report
                .components
                .iter()
                .filter(|c| c.status == anna_common::ComponentStatus::Critical)
                .collect();
            println!(
                "  {}  Critical: {}",
                "!".bright_red(),
                critical
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        OverallHealth::Unknown => {
            println!("  {}  Status unknown", "?".dimmed());
        }
    }
    println!("{}", THIN_SEPARATOR);
    println!();

    Ok(())
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
