//! Stats Command v5.7.2 - Daemon Activity with Top Offenders
//!
//! Shows Anna's daemon activity and background work.
//!
//! v5.7.2: Fixed uptime to use daemon API instead of broken systemctl parsing
//!
//! Sections:
//! - [DAEMON] Uptime, health status
//! - [LOG SCANNER] Scan cycles and timing
//! - [ERROR SUMMARY] Top offenders with counts, timestamps, samples

use anyhow::Result;
use owo_colors::OwoColorize;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    ErrorIndex, LogScanState, IntrusionIndex,
    format_duration_secs, format_time_ago, truncate_str,
};

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

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Daemon Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    // Load data
    let log_scan_state = LogScanState::load();
    let error_index = ErrorIndex::load();
    let intrusion_index = IntrusionIndex::load();

    // [DAEMON] - v5.7.2: now async, gets uptime from API
    print_daemon_section().await;

    // [LOG SCANNER]
    print_scanner_section(&log_scan_state);

    // [ERROR SUMMARY] with top offenders
    print_error_summary_section(&error_index, &intrusion_index);

    println!("{}", THIN_SEP);
    println!();
    println!("  'annactl status' for Anna's health.");
    println!("  'annactl knowledge' for what Anna knows.");
    println!();

    Ok(())
}

async fn print_daemon_section() {
    println!("{}", "[DAEMON]".cyan());

    // v5.7.2: Get uptime from daemon API (same as status command)
    match get_daemon_info().await {
        Some(info) => {
            let uptime_str = format_duration_secs(info.uptime_secs);
            println!("  Uptime:    {}", uptime_str);
            println!("  Objects:   {}", info.objects_tracked);
            println!("  Health:    {}", "healthy".green());
        }
        None => {
            println!("  Uptime:    {}", "n/a".dimmed());
            println!("  Health:    {}", "stopped".red());
        }
    }

    println!();
}

fn print_scanner_section(log_scan_state: &LogScanState) {
    println!("{}", "[LOG SCANNER]".cyan());
    println!("  {}", "(since daemon start)".dimmed());

    // Total scans
    println!("  Scans:     {}", log_scan_state.total_scans);

    // Last scan time
    if log_scan_state.last_scan_at > 0 {
        println!("  Last scan: {}", format_time_ago(log_scan_state.last_scan_at));
    } else {
        println!("  Last scan: n/a");
    }

    // Average scan interval (only if we have enough data)
    if log_scan_state.total_scans > 1 && log_scan_state.created_at > 0 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let total_time = now.saturating_sub(log_scan_state.created_at);
        let avg_interval = total_time / log_scan_state.total_scans;
        println!("  Interval:  ~{}", format_duration_secs(avg_interval));
    }

    // Scanner status
    let scanner_status = if log_scan_state.running {
        "running".green().to_string()
    } else {
        "idle".to_string()
    };
    println!("  Status:    {}", scanner_status);

    println!();
}

fn print_error_summary_section(
    error_index: &ErrorIndex,
    intrusion_index: &IntrusionIndex,
) {
    println!("{}", "[ERROR SUMMARY]".cyan());
    println!("  {}", "(last 24 hours)".dimmed());

    // Get 24h counts
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(86400);

    // Collect per-object error stats
    let mut object_stats: Vec<ErrorObjectStats> = Vec::new();
    let mut total_errors_24h = 0u64;
    let mut total_warnings_24h = 0u64;

    for (obj_name, obj) in &error_index.objects {
        let mut obj_errors = 0u64;
        let mut obj_warnings = 0u64;
        let mut last_error_ts = 0u64;
        let mut sample_message = String::new();

        for log in &obj.logs {
            if log.timestamp >= cutoff {
                if log.severity.is_error() {
                    obj_errors += 1;
                    total_errors_24h += 1;
                    if log.timestamp > last_error_ts {
                        last_error_ts = log.timestamp;
                        sample_message = log.message.clone();
                    }
                } else if log.severity == anna_common::LogSeverity::Warning {
                    obj_warnings += 1;
                    total_warnings_24h += 1;
                }
            }
        }

        if obj_errors > 0 || obj_warnings > 0 {
            object_stats.push(ErrorObjectStats {
                name: obj_name.clone(),
                errors: obj_errors,
                warnings: obj_warnings,
                last_error_ts,
                sample: sample_message,
            });
        }
    }

    // Sort by error count descending
    object_stats.sort_by(|a, b| b.errors.cmp(&a.errors));

    // Show summary
    if total_errors_24h == 0 && total_warnings_24h == 0 {
        println!("  {}", "No errors or warnings in last 24h".green());
    } else {
        println!("  Errors:    {}", total_errors_24h);
        println!("  Warnings:  {}", total_warnings_24h);

        // Show top offenders (up to 5)
        if !object_stats.is_empty() {
            println!();
            println!("  {}", "Top offenders:".bold());
            for stat in object_stats.iter().take(5) {
                let ts_str = if stat.last_error_ts > 0 {
                    format_time_ago(stat.last_error_ts)
                } else {
                    "n/a".to_string()
                };

                // Format: "object_name - N errors, last at TIME"
                println!(
                    "    {} - {} errors, last {}",
                    stat.name.cyan(),
                    stat.errors,
                    ts_str.dimmed()
                );

                // Show sample if available
                if !stat.sample.is_empty() {
                    let sample = truncate_str(&stat.sample, 50);
                    println!("      {}", sample.dimmed());
                }
            }

            if object_stats.len() > 5 {
                println!("    ({} more objects with errors)", object_stats.len() - 5);
            }
        }
    }

    // Intrusions detected (24h)
    let intrusions = intrusion_index.recent_high_severity(86400, 1).len();
    if intrusions > 0 {
        println!();
        println!("  {}", "[INTRUSIONS]".red().bold());
        println!("  Detected:  {}", intrusions.to_string().red());
    }

    println!();
}

/// Stats for error display
struct ErrorObjectStats {
    name: String,
    errors: u64,
    #[allow(dead_code)]
    warnings: u64,
    last_error_ts: u64,
    sample: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// v5.7.2: Get daemon info from API (same pattern as status.rs)
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
