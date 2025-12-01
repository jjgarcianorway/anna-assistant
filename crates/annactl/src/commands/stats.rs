//! Stats Command v5.5.0 - Daemon Activity with Top Offenders
//!
//! Shows Anna's daemon activity and background work.
//!
//! v5.5.0: Added top offending error sources with counts, timestamps, samples.
//!
//! Sections:
//! - [DAEMON] Uptime, health status
//! - [LOG SCANNER] Scan cycles and timing
//! - [ERROR SUMMARY] Top offenders with counts, timestamps, samples

use anyhow::Result;
use owo_colors::OwoColorize;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    ErrorIndex, LogScanState, TelemetryAggregates, IntrusionIndex,
    format_duration_secs, format_time_ago, truncate_str,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Daemon Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    // Load data
    let log_scan_state = LogScanState::load();
    let telemetry = TelemetryAggregates::load();
    let error_index = ErrorIndex::load();
    let intrusion_index = IntrusionIndex::load();

    // [DAEMON]
    print_daemon_section(&telemetry);

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

fn print_daemon_section(telemetry: &TelemetryAggregates) {
    println!("{}", "[DAEMON]".cyan());

    // Daemon uptime from systemd
    let uptime = get_daemon_uptime();
    println!("  Uptime:    {}", uptime);

    // Daemon start time
    if telemetry.daemon_start_at > 0 {
        println!("  Started:   {}", format_time_ago(telemetry.daemon_start_at));
    }

    // Health check status
    let health_status = check_daemon_health();
    let status_str = if health_status {
        "healthy".green().to_string()
    } else {
        "unhealthy".red().to_string()
    };
    println!("  Health:    {}", status_str);

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

fn get_daemon_uptime() -> String {
    let output = std::process::Command::new("systemctl")
        .args(["show", "annad", "--property=ActiveEnterTimestamp"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(ts_str) = stdout.strip_prefix("ActiveEnterTimestamp=") {
            let ts_str = ts_str.trim();
            if !ts_str.is_empty() && ts_str != "n/a" {
                // v5.5.1: Parse systemctl timestamp format "Mon 2025-12-01 13:50:39 CET"
                let parts: Vec<&str> = ts_str.split_whitespace().collect();
                if parts.len() >= 3 {
                    let date_time_str = format!("{} {}", parts[1], parts[2]);
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S") {
                        let start = dt.and_utc().timestamp() as u64;
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        if now > start {
                            return format_duration_secs(now - start);
                        }
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

fn check_daemon_health() -> bool {
    // Quick health check via HTTP
    let output = std::process::Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "http://127.0.0.1:7865/v1/health"])
        .output();

    if let Ok(output) = output {
        let code = String::from_utf8_lossy(&output.stdout);
        return code.trim() == "200";
    }

    false
}
