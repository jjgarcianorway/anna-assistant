//! Stats Command v5.2.4 - Anna's Behavior and History
//!
//! Shows Anna's behavior and evolution over time.
//! Independent of specific knowledge objects.
//!
//! Sections:
//! - [RUN HISTORY] Daemon uptime, first/last events
//! - [KNOWLEDGE GROWTH] Object counts and discovery latency
//! - [ERROR PIPELINE] Error/warning counts with top objects
//! - [INVENTORY PERFORMANCE] Scan intervals and timing

use anyhow::Result;
use owo_colors::OwoColorize;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    ErrorIndex, KnowledgeStore, LogScanState, TelemetryAggregates,
    format_duration_secs, format_time_ago,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    // Load data
    let store = KnowledgeStore::load();
    let error_index = ErrorIndex::load();
    let log_scan_state = LogScanState::load();
    let telemetry = TelemetryAggregates::load();

    // [RUN HISTORY]
    print_run_history_section(&telemetry);

    // [KNOWLEDGE GROWTH]
    print_knowledge_growth_section(&store);

    // [ERROR PIPELINE]
    print_error_pipeline_section(&error_index);

    // [INVENTORY PERFORMANCE]
    print_inventory_performance_section(&log_scan_state);

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_run_history_section(telemetry: &TelemetryAggregates) {
    println!("{}", "[RUN HISTORY]".cyan());

    // Q&A is disabled in this phase
    println!("  Total runs:       Q&A disabled");

    // Daemon uptime from systemd
    let uptime = get_daemon_uptime();
    println!("  Daemon uptime:    {}", uptime);

    // First and last telemetry events - use daemon_start_at and last_updated
    if telemetry.daemon_start_at > 0 {
        println!("  First event:      {}", format_time_ago(telemetry.daemon_start_at));
    } else {
        println!("  First event:      n/a");
    }

    if telemetry.last_updated > 0 {
        println!("  Last event:       {}", format_time_ago(telemetry.last_updated));
    } else {
        println!("  Last event:       n/a");
    }

    println!();
}

fn print_knowledge_growth_section(store: &KnowledgeStore) {
    println!("{}", "[KNOWLEDGE GROWTH]".cyan());

    let total_objects = store.total_objects();
    println!("  Objects known:        {}", total_objects);

    // Count objects with usage
    let objects_with_runs = store
        .objects
        .values()
        .filter(|o| o.usage_count > 0 || o.total_cpu_time_ms > 0)
        .count();
    println!("  Objects with runs:    {}", objects_with_runs);

    // Average discovery latency
    let avg_latency = calculate_avg_discovery_latency(store);
    if avg_latency > 0 {
        println!("  Avg discovery latency: {}", format_duration_secs(avg_latency));
    } else {
        println!("  Avg discovery latency: n/a");
    }

    // Last new object
    let last_new = get_last_new_object_time(store);
    if last_new > 0 {
        println!("  Last new object:  {}", format_time_ago(last_new));
    } else {
        println!("  Last new object:  n/a");
    }

    println!();
}

fn print_error_pipeline_section(error_index: &ErrorIndex) {
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

    println!("  Errors indexed (24h):   {}", errors_24h);
    println!("  Warnings indexed (24h): {}", warnings_24h);

    // Top error objects
    let top_errors = error_index.top_by_errors(5);
    if !top_errors.is_empty() {
        println!();
        println!("  Top error objects:");
        for (name, count) in &top_errors {
            println!("    {}: {} errors", name, count);
        }
    }

    println!();
}

fn print_inventory_performance_section(log_scan_state: &LogScanState) {
    println!("{}", "[INVENTORY PERFORMANCE]".cyan());

    println!("  Full inventory scans:  {}", log_scan_state.total_scans);

    // Time since last full pass
    if log_scan_state.last_scan_at > 0 {
        println!("  Last full scan:        {}", format_time_ago(log_scan_state.last_scan_at));
    } else {
        println!("  Last full scan:        n/a");
    }

    // Average scan interval (estimate from total scans and uptime)
    if log_scan_state.total_scans > 1 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let total_time = now.saturating_sub(log_scan_state.created_at);
        let avg_interval = total_time / log_scan_state.total_scans;
        println!("  Avg scan interval:     {}", format_duration_secs(avg_interval));
    } else {
        println!("  Avg scan interval:     n/a");
    }

    // Log scan interval (typically every minute)
    println!("  Log scan interval:     ~1m");

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

fn calculate_avg_discovery_latency(store: &KnowledgeStore) -> u64 {
    let latencies: Vec<u64> = store
        .objects
        .values()
        .filter_map(|o| {
            if o.first_seen_at > 0 && o.first_seen_at > store.created_at {
                Some(o.first_seen_at.saturating_sub(store.created_at))
            } else {
                None
            }
        })
        .collect();

    if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    }
}

fn get_last_new_object_time(store: &KnowledgeStore) -> u64 {
    store
        .objects
        .values()
        .map(|o| o.first_seen_at)
        .max()
        .unwrap_or(0)
}
