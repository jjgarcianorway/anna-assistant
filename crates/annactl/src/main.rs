//! Anna CLI (annactl) - User interface wrapper
//!
//! v0.3.0: Strict CLI with LLM-orchestrated help/version
//! v0.4.0: Update status in version/help output
//! v0.5.0: Natural language configuration, hardware-aware model selection
//! v0.6.0: ASCII-only sysadmin style, multi-round reliability refinement
//! v0.7.0: Self-health monitoring with auto-repair and REPL notifications
//! v0.8.0: Observability and debug logging with JSONL output
//!
//! Only these commands exist:
//!   - annactl "<question>"    Ask Anna anything
//!   - annactl                 Start interactive REPL
//!   - annactl -V | --version  Show version (via LLM)
//!   - annactl -h | --help     Show help (via LLM)

mod client;
mod llm_client;
mod orchestrator;
mod output;

use anna_common::{
    AnnaConfigV5, HardwareProfile, OverallHealth, RepairSafety,
    generate_request_id, init_logger, log_request, logging, self_health,
    LogComponent, LogEntry, LogLevel, RequestContext, RequestStatus,
    clear_current_request, set_current_request,
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

        // Version flags - route through LLM
        [flag] if flag == "-V" || flag == "--version" || flag == "version" => {
            run_version_via_llm().await
        }

        // Help flags - route through LLM
        [flag] if flag == "-h" || flag == "--help" || flag == "help" => {
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
        if matches!(
            input.to_lowercase().as_str(),
            "quit" | "exit" | "q" | ":q"
        ) {
            // v0.6.0: ASCII-only output
            println!("\nGoodbye.\n");
            break;
        }

        // Handle version/help in REPL too
        if matches!(input.to_lowercase().as_str(), "version" | "-v" | "--version") {
            run_version_via_llm().await?;
            continue;
        }
        if matches!(input.to_lowercase().as_str(), "help" | "-h" | "--help") {
            run_help_via_llm().await?;
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
        format!("Anna v{}", env!("CARGO_PKG_VERSION")).bright_white().bold()
    );
    println!("   Your intelligent Linux assistant\n");
}

/// Ask Anna a question - the core function
async fn run_ask(question: &str) -> Result<()> {
    // v0.8.0: Create request context with correlation ID
    let request_id = generate_request_id();
    let sanitized_query = logging::sanitize_query(question);
    let ctx = RequestContext::new(request_id.clone(), sanitized_query.clone());
    set_current_request(ctx);

    // Log request start
    logging::logger().write_daemon(
        &LogEntry::new(LogLevel::Debug, LogComponent::Request, "Processing question")
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
            &LogEntry::new(LogLevel::Error, LogComponent::Request, "Daemon not available")
                .with_request_id(&request_id),
        );

        // v0.6.0: ASCII-only error output
        eprintln!("[ERROR] Anna daemon is not running");
        eprintln!(
            "   Run: {} to start",
            "sudo systemctl start annad".cyan()
        );

        // Log and clear request context
        logging::with_current_request(|ctx| {
            ctx.set_result(0.0, RequestStatus::Failed);
            log_request(&ctx.to_trace());
        });
        clear_current_request();

        std::process::exit(1);
    }

    // Run orchestrator with stability check
    let result = orchestrator::process_question(question, &daemon).await?;

    // v0.8.0: Log request completion and trace
    logging::with_current_request(|ctx| {
        let status = if result.confidence >= 0.9 {
            RequestStatus::Ok
        } else if result.confidence >= 0.7 {
            RequestStatus::Degraded
        } else {
            RequestStatus::Failed
        };
        ctx.set_result(result.confidence, status);
        log_request(&ctx.to_trace());
    });

    // Log completion
    logging::logger().write_daemon(
        &LogEntry::new(LogLevel::Info, LogComponent::Request, "Request completed")
            .with_request_id(&request_id)
            .with_fields(serde_json::json!({
                "confidence": result.confidence,
                "sources_count": result.sources.len()
            })),
    );

    clear_current_request();

    // Output result
    output::display_response(&result);

    Ok(())
}

/// Version via LLM pipeline - Anna answers about herself
async fn run_version_via_llm() -> Result<()> {
    let daemon = client::DaemonClient::new();

    // Build internal question for version info
    let version_question = "What is your version? Report: mode, update status, LLM config, daemon status.";

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

    let help_question = "How do I use Anna? Show usage, natural language configuration, and examples.";

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
                    println!(
                        "   * {}: {}",
                        component.name.yellow(),
                        component.message
                    );
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
            println!(
                "{}  Self-health: {}",
                "[NOTE]".dimmed(),
                "unknown".dimmed()
            );
            println!();
        }
    }

    // Show auto-repairs that were executed
    if !report.repairs_executed.is_empty() {
        println!(
            "{}  Auto-repairs executed:",
            "[AUTO-REPAIR]".bright_green()
        );
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
        println!(
            "{}  Manual action required:",
            "[ACTION]".yellow()
        );
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
