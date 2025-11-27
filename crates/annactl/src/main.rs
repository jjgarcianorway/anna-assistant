//! Anna CLI (annactl) - User interface wrapper
//!
//! v0.3.0: Strict CLI with LLM-orchestrated help/version
//! v0.4.0: Update status in version/help output
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

use anna_common::{load_update_config, load_update_state};
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
    println!(
        "{}  Interactive mode. Type {} to exit.\n",
        "🗣️".cyan(),
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
            println!("\n{}  Goodbye!\n", "👋".bright_magenta());
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
            eprintln!("{}  Error: {}", "✗".red(), e);
        }
    }

    Ok(())
}

fn print_banner() {
    println!(
        "\n{}  {}",
        "🤖".bright_magenta(),
        format!("Anna v{}", env!("CARGO_PKG_VERSION")).bright_white().bold()
    );
    println!("   Your intelligent Linux assistant\n");
}

/// Ask Anna a question - the core function
async fn run_ask(question: &str) -> Result<()> {
    let daemon = client::DaemonClient::new();

    // Check daemon health
    if !daemon.is_healthy().await {
        eprintln!("{}  Anna daemon is not running", "✗".red());
        eprintln!(
            "   Run: {} to start",
            "sudo systemctl start annad".cyan()
        );
        std::process::exit(1);
    }

    // Run orchestrator with stability check
    let result = orchestrator::process_question(question, &daemon).await?;

    // Output result
    output::display_response(&result);

    Ok(())
}

/// Version via LLM pipeline - Anna answers about herself
async fn run_version_via_llm() -> Result<()> {
    let daemon = client::DaemonClient::new();

    // Build internal question for version info
    let version_question = "What is your version? Report: Anna version, channel, update status, daemon status, model name, and tool catalog count.";

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

    // Load update config and state
    let update_config = load_update_config();
    let update_state = load_update_state();

    // Process through orchestrator for consistent formatting
    let result = orchestrator::process_internal_query(
        version_question,
        &daemon,
        orchestrator::InternalQueryType::Version {
            version: env!("CARGO_PKG_VERSION").to_string(),
            daemon_status,
            probe_count,
            update_config,
            update_state,
        },
    )
    .await?;

    output::display_response(&result);
    Ok(())
}

/// Help via LLM pipeline - Anna explains how to use herself
async fn run_help_via_llm() -> Result<()> {
    let daemon = client::DaemonClient::new();

    let help_question = "How do I use Anna? Show usage, available commands, examples, and auto-update configuration.";

    // Load update config
    let update_config = load_update_config();

    // Process through orchestrator for consistent formatting
    let result = orchestrator::process_internal_query(
        help_question,
        &daemon,
        orchestrator::InternalQueryType::Help { update_config },
    )
    .await?;

    output::display_response(&result);
    Ok(())
}

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
