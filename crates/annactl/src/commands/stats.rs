//! Stats Command v6.0.0 - Daemon Activity Statistics
//!
//! v6.0.0: Grounded stats - uses real data sources
//!
//! Sections:
//! - [DAEMON] Uptime, health status
//! - [ACTIVITY] Real-time system activity from journalctl
//! - [ERRORS] Top error-producing services from journalctl

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::grounded::errors::{ErrorCounts, get_top_error_units};
use anna_common::format_duration_secs;

const THIN_SEP: &str = "------------------------------------------------------------";

#[derive(serde::Deserialize)]
struct HealthResponse {
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    phase: String,
    uptime_secs: u64,
    #[allow(dead_code)]
    objects_tracked: usize,
}

/// Run the stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Daemon Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    // [DAEMON]
    print_daemon_section().await;

    // [ERRORS] - from real journalctl
    print_errors_section();

    println!("{}", THIN_SEP);
    println!();
    println!("  'annactl status' for system status.");
    println!("  'annactl knowledge' for what's installed.");
    println!();

    Ok(())
}

async fn print_daemon_section() {
    println!("{}", "[DAEMON]".cyan());

    match get_daemon_info().await {
        Some(info) => {
            let uptime_str = format_duration_secs(info.uptime_secs);
            println!("  Status:    {} (up {})", "running".green(), uptime_str);
        }
        None => {
            println!("  Status:    {}", "stopped".red());
        }
    }

    println!();
}

fn print_errors_section() {
    println!("{}", "[ERRORS]".cyan());
    println!("  {}", "(source: journalctl, last 24h)".dimmed());

    let counts = ErrorCounts::query_24h();

    if counts.errors > 0 {
        println!("  Errors:    {}", counts.errors.to_string().red());
    } else {
        println!("  Errors:    {}", "0".green());
    }

    if counts.warnings > 0 {
        println!("  Warnings:  {}", counts.warnings.to_string().yellow());
    } else {
        println!("  Warnings:  0");
    }

    // Top error-producing units
    let top_units = get_top_error_units(24, 5);
    if !top_units.is_empty() {
        println!();
        println!("  {}", "Top offenders:".bold());
        for unit in &top_units {
            println!(
                "    {} - {} errors",
                unit.unit.cyan(),
                unit.error_count
            );
            if !unit.sample_message.is_empty() {
                println!("      {}", unit.sample_message.dimmed());
            }
        }
    }

    println!();
}

async fn get_daemon_info() -> Option<HealthResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let response = client
        .get("http://127.0.0.1:7865/v1/health")
        .send()
        .await
        .ok()?;

    response.json::<HealthResponse>().await.ok()
}
