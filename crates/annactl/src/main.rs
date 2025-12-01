//! Anna CLI (annactl) v5.3.0 - Telemetry Core
//!
//! Pure system intelligence - no LLM, no Q&A.
//!
//! Commands:
//! - annactl status           Anna's health and daemon status
//! - annactl stats            Daemon activity statistics
//! - annactl knowledge        Overview of what Anna knows
//! - annactl knowledge stats  Coverage and quality statistics
//! - annactl knowledge <name> Full object profile
//! - annactl reset            Clear all data and restart
//! - annactl version          Show version info
//! - annactl help             Show help info

mod commands;

use anna_common::AnnaConfig;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
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

    let _config = AnnaConfig::load();

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => run_help(),
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("stats") => commands::stats::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("knowledge") => commands::knowledge::run().await,
        [cmd, sub] if cmd.eq_ignore_ascii_case("knowledge") && sub.eq_ignore_ascii_case("stats") => {
            commands::knowledge_stats::run().await
        }
        [cmd, name] if cmd.eq_ignore_ascii_case("knowledge") => {
            commands::knowledge_detail::run(name).await
        }
        [cmd] if cmd.eq_ignore_ascii_case("reset") => run_reset().await,
        [flag] if flag == "-V" || flag == "--version" || flag.eq_ignore_ascii_case("version") => {
            run_version()
        }
        [flag] if flag == "-h" || flag == "--help" || flag.eq_ignore_ascii_case("help") => {
            run_help()
        }
        _ => run_unknown_command(&args),
    }
}

// ============================================================================
// Version Command
// ============================================================================

fn run_version() -> Result<()> {
    println!();
    println!("  annactl v{}", VERSION);
    println!("  Telemetry Core - Pure System Intelligence");
    println!();
    Ok(())
}

// ============================================================================
// Help Command
// ============================================================================

fn run_help() -> Result<()> {
    println!();
    println!("{}", format!("ANNA v{} - Telemetry Core", VERSION).bold());
    println!("{}", THIN_SEP);
    println!();
    println!("  Anna is a system intelligence daemon that tracks every");
    println!("  executable, package, and service on your Linux system.");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!("  annactl status              Daemon health and status");
    println!("  annactl stats               Daemon activity statistics");
    println!("  annactl knowledge           Knowledge overview by category");
    println!("  annactl knowledge stats     Coverage and quality statistics");
    println!("  annactl knowledge <name>    Full object profile");
    println!("  annactl reset               Clear all data and restart");
    println!("  annactl version             Show version info");
    println!("  annactl help                Show this help");
    println!();
    println!("{}", "WHAT ANNA TRACKS:".bold());
    println!("  - Commands on PATH (binaries, scripts)");
    println!("  - Packages with version history");
    println!("  - Systemd services (active/enabled/failed)");
    println!("  - Process activity (CPU/memory usage)");
    println!("  - Errors from system logs");
    println!("  - Intrusion detection patterns");
    println!();
    Ok(())
}

// ============================================================================
// Reset Command
// ============================================================================

async fn run_reset() -> Result<()> {
    println!();
    println!("{}", "[RESET]".yellow());
    println!();

    // Clear data directories
    let dirs_to_clear = [
        "/var/lib/anna/knowledge",
        "/var/lib/anna/telemetry",
    ];

    for dir in &dirs_to_clear {
        if std::path::Path::new(dir).exists() {
            match std::fs::remove_dir_all(dir) {
                Ok(_) => println!("  Cleared: {}", dir),
                Err(e) => println!("  Failed to clear {}: {}", dir, e),
            }
        }
    }

    // Clear state files
    let files_to_clear = [
        "/var/lib/anna/telemetry_state.json",
        "/var/lib/anna/log_scan_state.json",
    ];

    for file in &files_to_clear {
        if std::path::Path::new(file).exists() {
            match std::fs::remove_file(file) {
                Ok(_) => println!("  Removed: {}", file),
                Err(e) => println!("  Failed to remove {}: {}", file, e),
            }
        }
    }

    println!();
    println!("  Reset complete. Restart annad to begin fresh discovery.");
    println!();
    Ok(())
}

// ============================================================================
// Unknown Command
// ============================================================================

fn run_unknown_command(args: &[String]) -> Result<()> {
    println!();
    println!("{}", "[UNKNOWN COMMAND]".yellow());
    println!();
    println!("  '{}' is not a recognized command.", args.join(" "));
    println!();
    println!("  Run 'annactl help' for available commands.");
    println!();
    Ok(())
}
