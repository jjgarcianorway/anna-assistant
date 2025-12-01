//! Stats Command v6.1.0 - Daemon Telemetry Only
//!
//! v6.1.0: Pure daemon statistics - no host system noise
//! - Daemon status, uptime, memory
//! - Inventory scan history
//! - Request tracking
//! - Internal errors (Anna's own pipeline)
//!
//! This command answers: "How is Anna performing?"
//! NOT: "What errors are happening on the host?"

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::format_duration_secs;

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Daemon Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    match get_daemon_stats().await {
        Some(stats) => {
            // [DAEMON]
            print_daemon_section(&stats);

            // [INVENTORY]
            print_inventory_section(&stats);

            // [REQUESTS]
            print_requests_section(&stats);

            // [INTERNAL ERRORS]
            print_internal_errors_section(&stats);
        }
        None => {
            println!("{}", "[DAEMON]".cyan());
            println!("  Status:     {}", "stopped".red());
            println!();
            println!("  Start with: {}", "sudo systemctl start annad".cyan());
            println!();
        }
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_daemon_section(stats: &StatsResponse) {
    println!("{}", "[DAEMON]".cyan());

    let uptime_str = format_duration_secs(stats.uptime_secs);
    println!("  Status:     {}", "running".green());
    println!("  Uptime:     {}", uptime_str);
    println!("  PID:        {}", stats.pid);

    // Memory
    let rss_mib = stats.memory_rss_kb as f64 / 1024.0;
    let peak_mib = stats.memory_peak_kb as f64 / 1024.0;
    println!("  Memory:     {:.1} MiB (peak {:.1} MiB)", rss_mib, peak_mib);

    println!();
}

fn print_inventory_section(stats: &StatsResponse) {
    println!("{}", "[INVENTORY]".cyan());

    // Last scan time
    let last_scan_str = if let Some(secs_ago) = stats.last_scan_secs_ago {
        if secs_ago < 60 {
            format!("{}s ago", secs_ago)
        } else {
            format!("{}m ago", secs_ago / 60)
        }
    } else {
        "never".to_string()
    };
    println!("  Last scan:      {}", last_scan_str);

    // Scan duration
    if stats.last_scan_duration_ms > 0 {
        let duration_str = if stats.last_scan_duration_ms < 1000 {
            format!("{}ms", stats.last_scan_duration_ms)
        } else {
            format!("{:.1}s", stats.last_scan_duration_ms as f64 / 1000.0)
        };
        println!("  Scan duration:  {}", duration_str);
    }

    // Total scans
    println!("  Scans total:    {} (this boot)", stats.scan_count);

    // What was indexed
    println!("  Indexed:        {} cmds, {} pkgs, {} svcs",
             stats.commands_count, stats.packages_count, stats.services_count);

    println!();
}

fn print_requests_section(stats: &StatsResponse) {
    println!("{}", "[REQUESTS]".cyan());

    println!("  CLI requests:   {}", stats.cli_requests);

    if stats.cli_requests > 0 && stats.avg_response_ms > 0 {
        println!("  Avg response:   {} ms", stats.avg_response_ms);
    }

    println!();
}

fn print_internal_errors_section(stats: &StatsResponse) {
    println!("{}", "[INTERNAL ERRORS]".cyan());
    println!("  {}", "(Anna's own pipeline, not system logs)".dimmed());

    let errors = &stats.internal_errors;
    let total = errors.subprocess_failures + errors.parser_failures + errors.unknown_commands;

    if total == 0 {
        println!("  This boot:      {}", "none".green());
    } else {
        if errors.subprocess_failures > 0 {
            println!("  Subprocess failures: {}", errors.subprocess_failures.to_string().yellow());
        }
        if errors.parser_failures > 0 {
            println!("  Parser failures:     {}", errors.parser_failures.to_string().yellow());
        }
        if errors.unknown_commands > 0 {
            println!("  Unknown commands:    {}", errors.unknown_commands);
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
    memory_rss_kb: u64,
    memory_peak_kb: u64,
    commands_count: usize,
    packages_count: usize,
    services_count: usize,
    last_scan_secs_ago: Option<u64>,
    last_scan_duration_ms: u64,
    scan_count: u32,
    cli_requests: u64,
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
