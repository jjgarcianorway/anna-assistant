//! Status Command v6.1.0 - Anna Status Only
//!
//! v6.1.0: Pure Anna status - no host system noise
//! - Daemon health (running, uptime, PID)
//! - Inventory sync status
//! - Auto-update status
//! - Internal errors (Anna's own, not system)
//!
//! This command answers: "Is Anna healthy?"
//! NOT: "What's happening on the host system?"

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::config::{AnnaConfig, UpdateState};
use anna_common::format_duration_secs;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the status command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", THIN_SEP);
    println!();

    // [VERSION]
    print_version_section();

    // [DAEMON] - Anna's daemon health
    print_daemon_section().await;

    // [INVENTORY] - What Anna has indexed
    print_inventory_section().await;

    // [UPDATES] - Auto-update status
    print_updates_section();

    // [INTERNAL ERRORS] - Anna's own errors, not system
    print_internal_errors_section().await;

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

async fn print_daemon_section() {
    println!("{}", "[DAEMON]".cyan());

    match get_daemon_stats().await {
        Some(stats) => {
            let uptime_str = format_duration_secs(stats.uptime_secs);
            println!("  Status:     {}", "running".green());
            println!("  Uptime:     {}", uptime_str);
            println!("  PID:        {}", stats.pid);
        }
        None => {
            println!("  Status:     {}", "stopped".red());
            println!();
            println!("  Start with: {}", "sudo systemctl start annad".cyan());
        }
    }

    println!();
}

async fn print_inventory_section() {
    println!("{}", "[INVENTORY]".cyan());

    match get_daemon_stats().await {
        Some(stats) => {
            println!("  Packages:   {}  {}", stats.packages_count, "(from pacman -Q)".dimmed());
            println!("  Commands:   {}  {}", stats.commands_count, "(from $PATH)".dimmed());
            println!("  Services:   {}  {}", stats.services_count, "(from systemctl)".dimmed());

            // Sync status
            let sync_status = if let Some(secs_ago) = stats.last_scan_secs_ago {
                if secs_ago < 60 {
                    format!("{} (last scan {}s ago)", "OK".green(), secs_ago)
                } else if secs_ago < 300 {
                    format!("{} (last scan {}m ago)", "OK".green(), secs_ago / 60)
                } else {
                    format!("{} (last scan {}m ago)", "stale".yellow(), secs_ago / 60)
                }
            } else {
                "pending".yellow().to_string()
            };
            println!("  Sync:       {}", sync_status);
        }
        None => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

fn print_updates_section() {
    println!("{}", "[UPDATES]".cyan());

    let config = AnnaConfig::load();
    let state = UpdateState::load();

    // Mode
    let mode_str = if config.update.enabled {
        format!("{} (enabled)", "auto".green())
    } else {
        format!("{}", "disabled".yellow())
    };
    println!("  Mode:       {}", mode_str);

    // Interval
    println!("  Interval:   {}m", config.update.interval_minutes);

    // Last check
    println!("  Last check: {}", state.format_last_check());

    // Last result
    let result_display = if state.last_result.is_empty() {
        "n/a".to_string()
    } else if state.last_result.contains("success") || state.last_result.contains("up to date") {
        state.last_result.green().to_string()
    } else if state.last_result.contains("available") {
        state.last_result.cyan().to_string()
    } else {
        state.last_result.clone()
    };
    println!("  Result:     {}", result_display);

    // Show latest version if different from current
    if let Some(ref latest) = state.latest_version {
        if !state.current_version.is_empty() && latest != &state.current_version {
            println!("  Update:     {} → {}", state.current_version.dimmed(), latest.cyan());
        }
    }

    // Next check
    if config.update.enabled {
        println!("  Next check: {}", state.format_next_check());
    }

    println!();
}

async fn print_internal_errors_section() {
    println!("{}", "[INTERNAL ERRORS]".cyan());
    println!("  {}", "(Anna's own errors, not system logs)".dimmed());

    match get_daemon_stats().await {
        Some(stats) => {
            let errors = &stats.internal_errors;
            let total = errors.subprocess_failures + errors.parser_failures + errors.unknown_commands;

            if total == 0 {
                println!("  Last 24h:   {}", "none".green());
            } else {
                println!("  Last 24h:");
                if errors.subprocess_failures > 0 {
                    println!("    Command failures: {}", errors.subprocess_failures.to_string().yellow());
                }
                if errors.parser_failures > 0 {
                    println!("    Parse errors:     {}", errors.parser_failures.to_string().yellow());
                }
                if errors.unknown_commands > 0 {
                    println!("    Unknown commands: {}", errors.unknown_commands);
                }
            }
        }
        None => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

// ============================================================================
// Daemon API Client
// ============================================================================

#[derive(serde::Deserialize)]
struct StatsResponse {
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    version: String,
    uptime_secs: u64,
    pid: u32,
    #[allow(dead_code)]
    memory_rss_kb: u64,
    #[allow(dead_code)]
    memory_peak_kb: u64,
    commands_count: usize,
    packages_count: usize,
    services_count: usize,
    last_scan_secs_ago: Option<u64>,
    #[allow(dead_code)]
    last_scan_duration_ms: u64,
    #[allow(dead_code)]
    scan_count: u32,
    #[allow(dead_code)]
    cli_requests: u64,
    #[allow(dead_code)]
    avg_response_ms: u64,
    internal_errors: InternalErrors,
}

#[derive(serde::Deserialize)]
struct InternalErrors {
    subprocess_failures: u32,
    parser_failures: u32,
    unknown_commands: u32,
}

async fn get_daemon_stats() -> Option<StatsResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let response = client
        .get("http://127.0.0.1:7865/v1/stats")
        .send()
        .await
        .ok()?;

    response.json::<StatsResponse>().await.ok()
}
