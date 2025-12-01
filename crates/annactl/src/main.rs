//! Anna CLI (annactl) v5.2.4 - Knowledge Core
//!
//! Command separation by responsibility:
//! - annactl status       Anna's own health and status
//! - annactl stats        Anna's behavior and history statistics
//! - annactl knowledge    Overview of what Anna knows
//! - annactl knowledge stats    Detailed knowledge statistics
//! - annactl knowledge <name>   Full object profile
//! - annactl version      Show version info
//! - annactl help         Show help info
//!
//! Everything else returns a clear message that Q&A is disabled.

#![allow(dead_code)]
#![allow(unused_imports)]

mod client;
mod commands;

use anna_common::{init_logger, AnnaConfigV5, logging, LogComponent};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

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
    logging::logger().info(LogComponent::Request, "annactl v5 starting");

    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => run_help(),
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("stats") => commands::stats::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("knowledge") => commands::knowledge::run().await,
        [cmd, topic] if cmd.eq_ignore_ascii_case("knowledge") && topic.eq_ignore_ascii_case("stats") => {
            commands::knowledge_stats::run().await
        }
        [cmd, name] if cmd.eq_ignore_ascii_case("knowledge") => {
            commands::knowledge_detail::run(name).await
        }
        [flag] if flag == "-V" || flag == "--version" || flag.eq_ignore_ascii_case("version") => {
            run_version()
        }
        [flag] if flag.eq_ignore_ascii_case("-h") || flag.eq_ignore_ascii_case("--help") || flag.eq_ignore_ascii_case("help") => {
            run_help()
        }
        _ => run_disabled_message(),
    }
}

// ============================================================================
// Version Command
// ============================================================================

fn run_version() -> Result<()> {
    println!();
    println!("  annactl v{}", VERSION);
    println!("  Knowledge Core");
    println!();
    Ok(())
}

// ============================================================================
// Help Command
// ============================================================================

fn run_help() -> Result<()> {
    println!();
    println!("{}", "ANNA - Knowledge Core v5.2.4".bold());
    println!("{}", THIN_SEP);
    println!();
    println!("  Anna is a paranoid archivist that tracks every executable,");
    println!("  package, and service on your Linux system.");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!("  annactl status              Anna's health and status");
    println!("  annactl stats               Behavior and history statistics");
    println!("  annactl knowledge           Overview of knowledge by category");
    println!("  annactl knowledge stats     Detailed coverage statistics");
    println!("  annactl knowledge <name>    Full object profile");
    println!("  annactl version             Show version info");
    println!("  annactl help                Show this help");
    println!();
    println!("{}", "TRACKING:".bold());
    println!("  - ALL commands on PATH");
    println!("  - ALL packages with versions");
    println!("  - ALL systemd services (active/enabled/masked/failed)");
    println!("  - Package install/remove events");
    println!("  - Errors and warnings from journalctl");
    println!("  - Intrusion detection patterns");
    println!();
    Ok(())
}

// ============================================================================
// Disabled Q&A Message
// ============================================================================

fn run_disabled_message() -> Result<()> {
    println!();
    println!("{}", "[NOTICE]".yellow());
    println!();
    println!("  Q&A is disabled in Knowledge Core phase.");
    println!();
    println!("  Available commands:");
    println!("    annactl status           - Anna's health and status");
    println!("    annactl stats            - Behavior statistics");
    println!("    annactl knowledge        - Knowledge overview");
    println!("    annactl knowledge stats  - Coverage statistics");
    println!("    annactl knowledge <name> - Object details");
    println!("    annactl help             - Show all commands");
    println!();
    Ok(())
}
