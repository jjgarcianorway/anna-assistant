//! Anna CLI (annactl) v7.40.0 - Cache-First Software Discovery
//!
//! v7.40.0: Cache-first architecture for fast sw command
//! - sw uses persistent cache with delta detection
//! - p95 < 1.0s for sw when cache warm
//! - --full for detailed view, --json for machine output
//! - version subcommand for parseability
//!
//! v7.39.0: Terminal-adaptive rendering, domain status, checking indicator
//! - Compact mode for small terminals (< 24 rows or < 60 cols)
//! - Standard mode for normal terminals
//! - Wide mode for large terminals (> 120 cols)
//! - Shows "checking..." when domain refresh is in progress
//!
//! v7.38.0: Cache-only status, no live probing
//! - status reads status_snapshot.json only (no pacman, systemctl, journalctl)
//! - --version outputs exactly "vX.Y.Z" (no banners, no ANSI)
//!
//! Commands:
//! - annactl                  show help
//! - annactl --version        show version (also: annactl version)
//! - annactl status           health and runtime of Anna
//! - annactl sw               software overview (packages, commands, services)
//! - annactl sw --full        software overview with all sections
//! - annactl sw --json        software data in JSON format
//! - annactl sw NAME          software profile
//! - annactl hw               hardware overview
//! - annactl hw NAME          hardware profile

mod commands;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use anna_common::grounded::categoriser::is_valid_category;

const THIN_SEP: &str = "------------------------------------------------------------";

/// Check if name is a software category (using rule-based categoriser + services)
fn is_sw_category(name: &str) -> bool {
    // Special case: services is always a category
    if name.eq_ignore_ascii_case("services") || name.eq_ignore_ascii_case("service") {
        return true;
    }
    is_valid_category(name)
}

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
        // annactl (no args) - show help
        [] => run_help(),

        // annactl --version or annactl version (v7.40.0: both work)
        [cmd] if cmd == "--version" || cmd == "-V" || cmd == "version" => run_version(),

        // annactl status
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,

        // annactl sw (software overview - default compact)
        [cmd] if cmd.eq_ignore_ascii_case("sw") => commands::sw::run().await,

        // annactl sw --full (full detailed view)
        [cmd, flag] if cmd.eq_ignore_ascii_case("sw") && flag == "--full" => {
            commands::sw::run_full().await
        }

        // annactl sw --json (JSON output)
        [cmd, flag] if cmd.eq_ignore_ascii_case("sw") && flag == "--json" => {
            commands::sw::run_json().await
        }

        // annactl sw <name-or-category>
        [cmd, name] if cmd.eq_ignore_ascii_case("sw") => {
            if is_sw_category(name) {
                commands::sw_detail::run_category(name).await
            } else {
                commands::sw_detail::run_object(name).await
            }
        }

        // annactl hw (hardware overview)
        [cmd] if cmd.eq_ignore_ascii_case("hw") => commands::hw::run().await,

        // annactl hw <name>
        [cmd, name] if cmd.eq_ignore_ascii_case("hw") => {
            commands::hw_detail::run(name).await
        }

        // Unknown command
        _ => run_unknown(&args),
    }
}

/// Top-level help - concise list of commands
fn run_help() -> Result<()> {
    println!();
    println!("{}", "  Anna CLI".bold());
    println!("{}", THIN_SEP);
    println!("  annactl                  show this help");
    println!("  annactl --version        show version");
    println!("  annactl status           health and runtime of Anna");
    println!("  annactl sw               software overview (compact)");
    println!("  annactl sw --full        software overview (detailed)");
    println!("  annactl sw --json        software data (machine-readable)");
    println!("  annactl sw NAME          software profile");
    println!("  annactl hw               hardware overview");
    println!("  annactl hw NAME          hardware profile");
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

/// Print version - outputs "annactl vX.Y.Z"
/// v7.40.0: Changed format for better parseability
fn run_version() -> Result<()> {
    println!("annactl v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn run_unknown(args: &[String]) -> Result<()> {
    eprintln!();
    eprintln!("  {} '{}' is not a recognized command.", "error:".red(), args.join(" "));
    eprintln!();
    eprintln!("  Run 'annactl' for available commands.");
    eprintln!();
    std::process::exit(1);
}
