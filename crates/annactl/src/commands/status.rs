//! Status Command v5.3.0 - Daemon Health
//!
//! Shows Anna daemon health and system coverage.
//!
//! Sections:
//! - [VERSION] annactl/annad versions
//! - [DAEMON] Daemon state and uptime
//! - [INVENTORY] Scan progress (indexed / total)
//! - [HEALTH] Log pipeline summary (24h)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    ErrorIndex, ServiceIndex, IntrusionIndex, LogScanState,
    KnowledgeStore, count_path_binaries, count_systemd_services,
    format_duration_secs, format_time_ago, format_percent,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the status command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", THIN_SEP);
    println!();

    // Load data
    let store = KnowledgeStore::load();
    let error_index = ErrorIndex::load();
    let service_index = ServiceIndex::load();
    let log_scan_state = LogScanState::load();

    // [VERSION]
    print_version_section();

    // [DAEMON]
    print_daemon_section().await;

    // [INVENTORY]
    print_inventory_section(&store);

    // [HEALTH]
    print_health_section(&error_index, &service_index, &log_scan_state);

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  annactl:  v{}", VERSION);
    println!("  annad:    v{}", VERSION);
    println!();
}

async fn print_daemon_section() {
    println!("{}", "[DAEMON]".cyan());

    // Check daemon status via health endpoint
    let daemon_running = check_daemon_health().await;
    if daemon_running {
        let uptime = get_daemon_uptime();
        println!("  Status:   {} (up {})", "running".green(), uptime);
    } else {
        println!("  Status:   {}", "stopped".red());
    }

    // Check data directory
    let data_access = check_dir_access("/var/lib/anna");
    println!("  Data:     {}  /var/lib/anna", data_access);

    println!();
}

fn print_inventory_section(store: &KnowledgeStore) {
    println!("{}", "[INVENTORY]".cyan());
    println!("  {}", "(indexed / total on system)".dimmed());

    // Get counts
    let total_path_cmds = count_path_binaries();
    let total_services = count_systemd_services();
    let (commands, packages, services) = store.count_by_type();

    // Calculate overall progress
    let total_known = store.total_objects();
    let total_possible = total_path_cmds + total_services;
    let progress = if total_possible > 0 {
        (total_known as f64 / total_possible as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Determine status
    let status = if progress >= 99.0 {
        "complete".green().to_string()
    } else if progress > 0.0 {
        "scanning".yellow().to_string()
    } else {
        "waiting".to_string()
    };

    println!(
        "  Commands:   {}/{} ({})",
        commands,
        total_path_cmds,
        format_percent((commands as f64 / total_path_cmds.max(1) as f64) * 100.0)
    );
    println!("  Packages:   {}", packages);
    println!(
        "  Services:   {}/{} ({})",
        services,
        total_services,
        format_percent((services as f64 / total_services.max(1) as f64) * 100.0)
    );
    println!("  Status:     {}", status);

    println!();
}

fn print_health_section(
    error_index: &ErrorIndex,
    service_index: &ServiceIndex,
    log_scan_state: &LogScanState,
) {
    println!("{}", "[HEALTH]".cyan());
    println!("  {}", "(last 24 hours)".dimmed());

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

    // Get intrusion count (24h)
    let intrusion_index = IntrusionIndex::load();
    let intrusions = intrusion_index.recent_high_severity(86400, 1).len();

    // Get failed services count
    let failed_services = service_index.failed_count;

    println!("  Errors:      {}", errors_24h);
    println!("  Warnings:    {}", warnings_24h);

    if intrusions > 0 {
        println!("  Intrusions:  {}", intrusions.to_string().red());
    }

    if failed_services > 0 {
        println!("  Failed svcs: {}", failed_services.to_string().red());
    }

    // Log scanner status
    let scanner_status = if log_scan_state.running {
        "running".green().to_string()
    } else {
        "idle".to_string()
    };
    let last_scan = format_time_ago(log_scan_state.last_scan_at);
    println!("  Scanner:     {} (last {})", scanner_status, last_scan);

    println!();
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn check_daemon_health() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    client
        .get("http://127.0.0.1:7865/v1/health")
        .send()
        .await
        .is_ok()
}

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

fn check_dir_access(path: &str) -> String {
    let p = Path::new(path);
    if !p.exists() {
        return "---".to_string();
    }

    let readable = fs::read_dir(p).is_ok();
    let writable = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(p.join(".anna_test"))
        .map(|_| {
            let _ = fs::remove_file(p.join(".anna_test"));
            true
        })
        .unwrap_or(false);

    match (readable, writable) {
        (true, true) => "R/W".green().to_string(),
        (true, false) => "R/-".yellow().to_string(),
        (false, true) => "-/W".yellow().to_string(),
        (false, false) => "---".red().to_string(),
    }
}
