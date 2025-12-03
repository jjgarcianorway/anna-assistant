//! Anna CLI (annactl) v0.0.23 - Self-Sufficiency
//!
//! Public CLI surface (strict):
//! - annactl                  REPL mode (interactive)
//! - annactl <request...>     one-shot natural language request
//! - annactl status           self-status
//! - annactl reset            factory reset (requires root)
//! - annactl uninstall        complete removal (requires root)
//! - annactl --version        version (also: -V)
//!
//! All other commands route through natural language processing.
//! Internal capabilities (sw, hw, snapshots) are accessed via requests.
//!
//! v0.0.12: Proactive Anomaly Detection
//! - Alert surfacing in REPL: "New alerts since last session"
//! - Alert footer in one-shot mode: "Active alerts" summary
//! - what_changed and slowness_hypotheses tools
//! - Evidence IDs in all alerts
//!
//! v0.0.11: Safe Auto-Update System
//! - Update channels: stable (default) and canary
//! - Safe update workflow: download, verify, stage, atomic install, restart
//! - Guardrails: disk space, mutation lock, installer review
//! - Zero-downtime restart via systemd
//! - Automatic rollback on failure
//! - Complete state visibility in annactl status
//!
//! Multi-party dialogue transcript:
//! [you] -> [anna] -> [translator] -> [anna] -> [annad] -> [anna] -> [junior] -> [anna] -> [you]

mod commands;
mod pipeline;

use anna_common::{AnnaConfig, OllamaClient, select_junior_model, AlertQueue, AnomalySeverity};
use anna_common::model_selection::{BootstrapState, BootstrapPhase};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use std::io::{self, Write};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const THIN_SEP: &str = "------------------------------------------------------------";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .init();

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        // annactl (no args) - REPL mode
        [] => run_repl().await,

        // annactl --version or -V
        [cmd] if cmd == "--version" || cmd == "-V" => run_version(),

        // annactl --help or -h (show help, not REPL)
        [cmd] if cmd == "--help" || cmd == "-h" => run_help(),

        // annactl status - self-status
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,

        // annactl reset - factory reset (requires root)
        [cmd] if cmd.eq_ignore_ascii_case("reset") => {
            commands::reset::run(commands::reset::ResetOptions::default()).await
        }
        [cmd, flag] if cmd.eq_ignore_ascii_case("reset") && (flag == "--dry-run") => {
            commands::reset::run(commands::reset::ResetOptions { dry_run: true, force: false }).await
        }
        [cmd, flag] if cmd.eq_ignore_ascii_case("reset") && (flag == "--force" || flag == "-f") => {
            commands::reset::run(commands::reset::ResetOptions { dry_run: false, force: true }).await
        }

        // annactl uninstall - complete removal (requires root)
        [cmd] if cmd.eq_ignore_ascii_case("uninstall") => {
            commands::uninstall::run(commands::uninstall::UninstallOptions::default()).await
        }
        [cmd, flag] if cmd.eq_ignore_ascii_case("uninstall") && (flag == "--dry-run") => {
            commands::uninstall::run(commands::uninstall::UninstallOptions { dry_run: true, force: false, keep_helpers: false }).await
        }
        [cmd, flag] if cmd.eq_ignore_ascii_case("uninstall") && (flag == "--force" || flag == "-f") => {
            commands::uninstall::run(commands::uninstall::UninstallOptions { dry_run: false, force: true, keep_helpers: false }).await
        }
        [cmd, flag] if cmd.eq_ignore_ascii_case("uninstall") && (flag == "--keep-helpers") => {
            commands::uninstall::run(commands::uninstall::UninstallOptions { dry_run: false, force: false, keep_helpers: true }).await
        }

        // Everything else is a natural language request
        _ => run_request(&args.join(" ")).await,
    }
}

/// Check and display LLM bootstrap status - v0.0.22: use BootstrapState
async fn check_junior_status() -> Option<String> {
    let config = AnnaConfig::load();
    let bootstrap = BootstrapState::load();

    // Check bootstrap phase for informative messages
    match bootstrap.phase {
        BootstrapPhase::Ready => {
            // All good - show selected models
            if let Some(ref translator) = bootstrap.translator {
                println!("  {} Translator: {}", "[*]".green(), translator.model);
            }
            if let Some(ref junior) = bootstrap.junior {
                println!("  {} Junior LLM: {}", "[*]".green(), junior.model);
                return Some(junior.model.clone());
            }
        }
        BootstrapPhase::DetectingOllama => {
            println!("  {} Anna is starting up, checking Ollama...", "[~]".cyan());
            println!("      Please wait a moment for LLM setup.");
        }
        BootstrapPhase::InstallingOllama => {
            println!("  {} Anna is installing Ollama (this may take a few minutes)...", "[~]".cyan());
            println!("      The daemon is setting up LLM infrastructure.");
        }
        BootstrapPhase::PullingModels => {
            if let Some(ref progress) = bootstrap.download_progress {
                println!("  {} Downloading model: {} ({:.1}%)",
                    "[~]".cyan(),
                    progress.model,
                    progress.percent());
                if let Some(eta) = progress.eta_seconds {
                    let eta_str = if eta < 60 {
                        format!("{}s", eta)
                    } else if eta < 3600 {
                        format!("{}m {}s", eta / 60, eta % 60)
                    } else {
                        format!("{}h {}m", eta / 3600, (eta % 3600) / 60)
                    };
                    println!("      Estimated time remaining: {}", eta_str);
                }
            } else {
                println!("  {} Anna is downloading LLM models...", "[~]".cyan());
            }
            println!("      Please come back after download completes.");
        }
        BootstrapPhase::Benchmarking => {
            println!("  {} Anna is benchmarking models to find the best fit...", "[~]".cyan());
        }
        BootstrapPhase::Error => {
            if let Some(ref err) = bootstrap.error {
                if err.contains("not available") {
                    println!("  {} Ollama not available", "[!]".yellow());
                    println!("      Anna's daemon will auto-install Ollama when possible.");
                    println!("      Or install manually: curl -fsSL https://ollama.ai/install.sh | sh");
                } else {
                    println!("  {} LLM setup error: {}", "[!]".yellow(), err);
                }
            } else {
                println!("  {} LLM setup encountered an error", "[!]".yellow());
            }
            println!("      Responses will use deterministic fallback (lower quality).");
        }
    }

    // Fallback: check Ollama directly if bootstrap not ready
    if !config.junior.enabled {
        println!("  {} Junior LLM disabled in config", "[!]".yellow());
        return None;
    }

    let client = OllamaClient::with_url(&config.junior.ollama_url);
    if !client.is_available().await {
        return None;
    }

    // Get model from available list
    if let Ok(models) = client.list_models().await {
        let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
        return select_junior_model(&model_names);
    }

    None
}

/// REPL mode - interactive natural language chat
async fn run_repl() -> Result<()> {
    println!();
    let version = env!("CARGO_PKG_VERSION");
    println!("{}", format!("  Anna Assistant v{}", version).bold());
    println!("{}", THIN_SEP);
    println!("  Natural language interface to your system.");

    // Check Junior status
    check_junior_status().await;

    // v0.0.12: Show alerts on REPL start
    show_alerts_summary();

    println!("  Type your question or request. Type 'exit' to quit.");
    println!("{}", THIN_SEP);
    println!();

    loop {
        print!("  {} ", "you>".bright_white().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit")
            || input.eq_ignore_ascii_case("quit")
            || input.eq_ignore_ascii_case("bye")
            || input == "q"
        {
            println!();
            println!("  Goodbye!");
            println!();
            break;
        }

        if input.eq_ignore_ascii_case("help") {
            print_repl_help();
            continue;
        }

        if input.eq_ignore_ascii_case("status") {
            commands::status::run().await?;
            continue;
        }

        // Process as natural language request
        process_request(input).await;
    }

    Ok(())
}

/// One-shot natural language request
async fn run_request(request: &str) -> Result<()> {
    println!();
    process_request(request).await;

    // v0.0.12: Show alert footer in one-shot mode
    show_alerts_footer();

    println!();
    Ok(())
}

/// Process a natural language request through the pipeline
async fn process_request(request: &str) {
    // v0.0.3: Full multi-party dialogue pipeline
    // [you] -> [anna] -> [translator] -> [anna] -> [annad] -> [anna] -> [junior] -> [anna] -> [you]
    pipeline::process(request).await;
}

/// v0.0.12: Show alerts summary for REPL welcome
fn show_alerts_summary() {
    let queue = AlertQueue::load();
    let (critical, warning, info) = queue.count_by_severity();
    let total = critical + warning + info;

    if total == 0 {
        return; // No alerts, nothing to show
    }

    println!();
    if critical > 0 {
        println!("  {} {} critical alert(s) active!",
            "[!]".red().bold(),
            critical.to_string().red().bold());
    }
    if warning > 0 {
        println!("  {} {} warning(s) active",
            "[!]".yellow(),
            warning.to_string().yellow());
    }
    if info > 0 && critical == 0 && warning == 0 {
        println!("  {} {} info alert(s)",
            "[i]".dimmed(),
            info);
    }

    // Show latest alert
    let active = queue.get_active();
    if let Some(latest) = active.first() {
        println!("      Latest: [{}] {}", latest.evidence_id.cyan(), latest.title);
    }

    println!("      Run 'status' for details.");
}

/// v0.0.12: Show alerts footer for one-shot mode
fn show_alerts_footer() {
    let queue = AlertQueue::load();
    let (critical, warning, _info) = queue.count_by_severity();

    // Only show if there are critical or warning alerts
    if critical == 0 && warning == 0 {
        return;
    }

    println!();
    println!("{}", THIN_SEP);

    if critical > 0 {
        println!("  {} {} critical, {} warning alert(s) active. Run 'annactl status' for details.",
            "[!]".red().bold(),
            critical.to_string().red().bold(),
            warning);
    } else {
        println!("  {} {} warning alert(s) active. Run 'annactl status' for details.",
            "[!]".yellow(),
            warning.to_string().yellow());
    }
}

/// Print REPL help
fn print_repl_help() {
    println!();
    println!("  {}", "Commands:".bold());
    println!("    exit, quit, bye, q  - leave REPL");
    println!("    status              - show Anna's status");
    println!("    help                - show this help");
    println!();
    println!("  {}", "Examples:".bold());
    println!("    what CPU do I have?");
    println!("    is nginx running?");
    println!("    show me disk usage");
    println!();
}

/// Print version
fn run_version() -> Result<()> {
    println!("annactl v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Print help (--help flag)
fn run_help() -> Result<()> {
    println!();
    println!("{}", "  Anna Assistant".bold());
    println!("{}", THIN_SEP);
    println!("  annactl                  interactive mode (REPL)");
    println!("  annactl <request>        one-shot natural language request");
    println!("  annactl status           show Anna's status");
    println!("  annactl reset            factory reset (requires root)");
    println!("  annactl uninstall        complete removal (requires root)");
    println!("  annactl --version        show version");
    println!("{}", THIN_SEP);
    println!();
    println!("  {}", "Reset options:".bold());
    println!("    --dry-run              show what would be deleted");
    println!("    --force, -f            skip confirmation prompt");
    println!();
    println!("  {}", "Uninstall options:".bold());
    println!("    --dry-run              show what would be deleted");
    println!("    --force, -f            skip confirmation prompt");
    println!("    --keep-helpers         don't offer to remove helpers");
    println!();
    println!("  {}", "Examples:".bold());
    println!("    annactl \"what CPU do I have?\"");
    println!("    annactl \"is docker running?\"");
    println!("    annactl \"show me recent errors\"");
    println!();
    Ok(())
}
