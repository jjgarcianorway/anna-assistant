//! Anna CLI (annactl) v7.42.1 - Inline Diagnostics
//!
//! ARCHITECTURE RULE: annactl NEVER does heavyweight scanning.
//! All data comes from snapshots written by annad daemon.
//!
//! v7.42.1: Inline Diagnostics
//! - status command now includes all diagnostic checks inline
//! - Removed separate doctor command (status is self-sufficient)
//! - [PATHS] section shows writable/not-writable status
//!
//! v7.42.0: Daemon/CLI Contract Fix
//! - status shows DAEMON (socket/systemd) and SNAPSHOT (file) separately
//! - Never conflate "no snapshot" with "daemon stopped"
//! - Control socket check for authoritative daemon health
//!
//! Commands:
//! - annactl                  show help
//! - annactl --version        show version (also: annactl version)
//! - annactl status           health, diagnostics, and runtime
//! - annactl sw               software overview (compact)
//! - annactl sw --full        software overview (all sections)
//! - annactl sw --json        software data (JSON)
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

        // annactl status (v7.42.1: includes inline diagnostics, no separate doctor command)
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,

        // annactl reset (v7.42.5: factory reset)
        [cmd] if cmd.eq_ignore_ascii_case("reset") => {
            commands::reset::run(commands::reset::ResetOptions::default()).await
        }

        // annactl reset --dry-run (show what would happen)
        [cmd, flag] if cmd.eq_ignore_ascii_case("reset") && flag == "--dry-run" => {
            commands::reset::run(commands::reset::ResetOptions { dry_run: true, force: false }).await
        }

        // annactl reset --force (skip confirmation)
        [cmd, flag] if cmd.eq_ignore_ascii_case("reset") && (flag == "--force" || flag == "-f") => {
            commands::reset::run(commands::reset::ResetOptions { dry_run: false, force: true }).await
        }

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
    println!("  annactl status           health, diagnostics, and runtime");
    println!("  annactl reset            factory reset (requires root)");
    println!("  annactl sw               software overview");
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
