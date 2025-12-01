//! Anna CLI (annactl) v5.6.0 - Telemetry Core
//!
//! Pure system intelligence - no LLM, no Q&A.
//! annactl is a REMOTE CONTROL - it never writes data directly.
//! All state mutations happen via RPC to annad (running as root).
//!
//! Commands:
//! - annactl status               Anna's health and daemon status
//! - annactl stats                Daemon activity statistics
//! - annactl knowledge            Overview of what Anna knows
//! - annactl knowledge stats      Coverage and quality statistics
//! - annactl knowledge <category> List objects in category (editors, shells, etc)
//! - annactl knowledge <name>     Full object profile
//! - annactl reset                Clear all data via daemon RPC
//! - annactl version              Show version and install info
//! - annactl help                 Show help info

mod commands;

use anna_common::AnnaConfig;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// v5.5.0: Known category names for `knowledge <category>` queries
const CATEGORY_NAMES: [&str; 8] = [
    "editors", "terminals", "shells", "browsers",
    "compositors", "wms", "tools", "services"
];

fn is_category_query(name: &str) -> bool {
    let lower = name.to_lowercase();
    CATEGORY_NAMES.contains(&lower.as_str())
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

    let _config = AnnaConfig::load();

    let args: Vec<String> = env::args().skip(1).collect();

    // v5.6.0: Simplified command routing - removed cpu/ram/disk magic commands
    match args.as_slice() {
        [] => run_help(),
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("stats") => commands::stats::run().await,
        [cmd] if cmd.eq_ignore_ascii_case("knowledge") => commands::knowledge::run().await,
        [cmd, sub] if cmd.eq_ignore_ascii_case("knowledge") && sub.eq_ignore_ascii_case("stats") => {
            commands::knowledge_stats::run().await
        }
        // v5.5.0: Category queries like `knowledge editors`
        [cmd, name] if cmd.eq_ignore_ascii_case("knowledge") && is_category_query(name) => {
            commands::knowledge_category::run(name).await
        }
        [cmd, name] if cmd.eq_ignore_ascii_case("knowledge") => {
            commands::knowledge_detail::run(name).await
        }
        [cmd] if cmd.eq_ignore_ascii_case("reset") => run_reset().await,
        [flag] if flag == "-V" || flag == "--version" || flag.eq_ignore_ascii_case("version") => {
            commands::version::run()
        }
        [flag] if flag == "-h" || flag == "--help" || flag.eq_ignore_ascii_case("help") => {
            run_help()
        }
        _ => run_unknown_command(&args),
    }
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
    println!("  annactl reset               Clear Anna's state (not system logs)");
    println!("  annactl version             Show version info");
    println!("  annactl help                Show this help");
    println!();
    println!("{}", "CATEGORY QUERIES:".bold());
    println!("  annactl knowledge editors   List installed editors");
    println!("  annactl knowledge shells    List installed shells");
    println!("  annactl knowledge terminals List installed terminals");
    println!("  annactl knowledge browsers  List installed browsers");
    println!("  annactl knowledge services  List indexed services");
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
// Reset Command v6.0.2 - Clarified Semantics
// ============================================================================

/// Response from the daemon reset endpoint
#[derive(serde::Deserialize)]
struct ResetResponse {
    success: bool,
    message: String,
    cleared_items: usize,
    errors: Vec<String>,
}

/// Request body for reset
#[derive(serde::Serialize)]
struct ResetRequest {
    clear_logs: bool,
}

async fn run_reset() -> Result<()> {
    println!();
    println!("{}", "[RESET]".yellow());
    println!();

    // v6.0.2: Clarify what reset actually does
    println!("  {}  What reset clears:", "INFO".cyan());
    println!("    - Anna's internal state (knowledge index, cached data)");
    println!("    - Anna's event logs (/var/lib/anna/events/)");
    println!("    - Update check state");
    println!();
    println!("  {}  What reset does NOT touch:", "INFO".cyan());
    println!("    - System logs (journalctl) - those are system-wide");
    println!("    - Installed packages - use pacman for that");
    println!("    - Running services - use systemctl for that");
    println!();

    // v5.6.0: Reset is done via RPC to the daemon (running as root)
    // No sudo required - annactl just sends the request
    println!("  {} Sending reset request to daemon...", "*".blue());

    let client = reqwest::Client::new();
    let request = ResetRequest { clear_logs: true };

    match client
        .post("http://127.0.0.1:7865/v1/reset")
        .json(&request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ResetResponse>().await {
                    Ok(result) => {
                        println!();
                        if result.success {
                            println!("  {}  {}", "OK".green(), result.message);
                            println!();
                            println!("  Cleared {} Anna state items.", result.cleared_items);
                        } else {
                            println!("  {}  {}", "WARN".yellow(), result.message);
                            if !result.errors.is_empty() {
                                println!();
                                println!("  Errors:");
                                for err in &result.errors {
                                    println!("    - {}", err.red());
                                }
                            }
                        }
                        println!();
                        println!("  Anna will re-scan the system on next status check.");
                        println!();
                    }
                    Err(e) => {
                        println!("  {}  Failed to parse response: {}", "ERROR".red(), e);
                        println!();
                        std::process::exit(1);
                    }
                }
            } else {
                println!("  {}  Daemon returned error: {}", "ERROR".red(), response.status());
                println!();
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("  {}  Cannot connect to daemon.", "ERROR".red());
            println!();
            println!("  {}", e.to_string().dimmed());
            println!();
            println!("  Is annad running? Check with:");
            println!("    {}", "systemctl status annad".cyan());
            println!();
            std::process::exit(1);
        }
    }

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
