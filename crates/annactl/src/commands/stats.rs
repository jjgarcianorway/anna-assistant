//! Stats Command v5.2.5 - Daemon Activity Only
//!
//! Shows Anna's daemon activity and background work.
//! Not per-object knowledge - just daemon behavior.
//!
//! Sections:
//! - [DAEMON ACTIVITY] Uptime, starts, health checks
//! - [INVENTORY CYCLES] Scan progress and timing
//! - [ERROR PIPELINE] Log scanning stats

use anyhow::Result;
use owo_colors::OwoColorize;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    ErrorIndex, LogScanState, TelemetryAggregates, IntrusionIndex,
    format_duration_secs, format_time_ago,
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

    // [DAEMON ACTIVITY]
    print_daemon_activity_section(&telemetry);

    // [INVENTORY CYCLES]
    print_inventory_cycles_section(&log_scan_state);

    // [ERROR PIPELINE]
    print_error_pipeline_section(&error_index, &log_scan_state, &intrusion_index);

    println!("{}", THIN_SEP);
    println!();
    println!("  Use 'annactl status' for Anna's health.");
    println!("  Use 'annactl knowledge' for what Anna knows.");
    println!();

    Ok(())
}

fn print_daemon_activity_section(telemetry: &TelemetryAggregates) {
    println!("{}", "[DAEMON ACTIVITY]".cyan());

    // Daemon uptime from systemd
    let uptime = get_daemon_uptime();
    println!("  Uptime:         {}", uptime);

    // Daemon start time
    if telemetry.daemon_start_at > 0 {
        println!("  Started:        {}", format_time_ago(telemetry.daemon_start_at));
    } else {
        println!("  Started:        n/a");
    }

    // Last activity
    if telemetry.last_updated > 0 {
        println!("  Last activity:  {}", format_time_ago(telemetry.last_updated));
    } else {
        println!("  Last activity:  n/a");
    }

    // Health check status
    let health_status = check_daemon_health();
    let status_str = if health_status {
        "healthy".green().to_string()
    } else {
        "unhealthy".red().to_string()
    };
    println!("  Health:         {}", status_str);

    println!();
}

fn print_inventory_cycles_section(log_scan_state: &LogScanState) {
    println!("{}", "[INVENTORY CYCLES]".cyan());

    // Total scans
    println!("  Total scans:    {}", log_scan_state.total_scans);

    // Last scan time
    if log_scan_state.last_scan_at > 0 {
        println!("  Last scan:      {}", format_time_ago(log_scan_state.last_scan_at));
    } else {
        println!("  Last scan:      n/a");
    }

    // Average scan interval
    if log_scan_state.total_scans > 1 && log_scan_state.created_at > 0 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let total_time = now.saturating_sub(log_scan_state.created_at);
        let avg_interval = total_time / log_scan_state.total_scans;
        println!("  Avg interval:   {}", format_duration_secs(avg_interval));
    } else {
        println!("  Avg interval:   n/a");
    }

    // Scanner status
    let scanner_status = if log_scan_state.running {
        "running".green().to_string()
    } else {
        "idle".to_string()
    };
    println!("  Scanner:        {}", scanner_status);

    println!();
}

fn print_error_pipeline_section(
    error_index: &ErrorIndex,
    log_scan_state: &LogScanState,
    intrusion_index: &IntrusionIndex,
) {
    println!("{}", "[ERROR PIPELINE]".cyan());

    // Get 24h counts
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(86400);

    let mut errors_24h = 0u64;
    let mut warnings_24h = 0u64;
    for obj in error_index.objects.values() {
        for log in &obj.logs {
            if log.timestamp >= cutoff {
                if log.severity.is_error() {
                    errors_24h += 1;
                } else if log.severity == anna_common::LogSeverity::Warning {
                    warnings_24h += 1;
                }
            }
        }
    }

    println!("  Errors (24h):      {}", errors_24h);
    println!("  Warnings (24h):    {}", warnings_24h);

    // New errors/warnings since last scan
    println!(
        "  New since scan:    +{} errors, +{} warnings",
        log_scan_state.new_errors, log_scan_state.new_warnings
    );

    // Intrusions detected
    let intrusions = intrusion_index.recent_high_severity(86400, 1).len();
    if intrusions > 0 {
        println!(
            "  Intrusions (24h):  {} detected",
            intrusions.to_string().red()
        );
    } else {
        println!("  Intrusions (24h):  0");
    }

    // Objects with errors
    let objects_with_errors = error_index.objects.len();
    println!("  Objects affected:  {}", objects_with_errors);

    println!();
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
                if let Ok(dt) = chrono::DateTime::parse_from_str(
                    &format!("{} +0000", ts_str),
                    "%a %Y-%m-%d %H:%M:%S %Z %z",
                ) {
                    let start = dt.timestamp() as u64;
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
