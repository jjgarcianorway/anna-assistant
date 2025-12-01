//! Status Command v5.2.4 - Anna's Health Only
//!
//! Shows a sysadmin a quick view of Anna's own health and background work.
//! No knowledge counts, no telemetry counters - just system health.
//!
//! Sections:
//! - [VERSION] annactl/annad versions
//! - [SERVICES] Daemon state and uptime
//! - [PERMISSIONS] Directory access status
//! - [UPDATES] Auto-update state
//! - [INVENTORY] Scan progress with ETA
//! - [HEALTH] Error pipeline summary

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

    // [SERVICES]
    print_services_section().await;

    // [PERMISSIONS]
    print_permissions_section();

    // [UPDATES]
    print_updates_section();

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

async fn print_services_section() {
    println!("{}", "[SERVICES]".cyan());

    // Check daemon status
    let daemon_status = check_daemon_status().await;
    if daemon_status.running {
        let uptime = get_daemon_uptime();
        println!("  Daemon:   {} (up {})", "running".green(), uptime);
    } else {
        println!("  Daemon:   {}", "not running".red());
    }

    // Check ollama status (if relevant)
    let ollama_running = check_ollama_status().await;
    if ollama_running {
        println!("  Ollama:   {}", "running".green());
    } else {
        println!("  Ollama:   {}", "not running".yellow());
    }

    println!();
}

fn print_permissions_section() {
    println!("{}", "[PERMISSIONS]".cyan());

    let dirs = [
        ("Data dir", "/var/lib/anna"),
        ("XP dir", "/var/lib/anna/xp"),
        ("Knowledge", "/var/lib/anna/knowledge"),
        ("LLM state", "/var/lib/anna/llm"),
    ];

    for (label, path) in &dirs {
        let access = check_dir_access(path);
        println!("  {:<12} {}  {}", format!("{}:", label), access, path);
    }

    println!();
}

fn print_updates_section() {
    println!("{}", "[UPDATES]".cyan());

    // Check auto-update config
    let config = anna_common::AnnaConfigV5::load();
    let auto_update = if config.update.enabled {
        let mins = config.update.interval_seconds / 60;
        format!("enabled (every {}m)", mins)
    } else {
        "disabled".to_string()
    };

    println!("  Auto-update:  {}", auto_update);

    // Last check time - read from state file if exists
    let last_check = get_last_update_check();
    println!("  Last check:   {}", last_check);

    println!();
}

fn print_inventory_section(store: &KnowledgeStore) {
    println!("{}", "[INVENTORY]".cyan());

    // Get counts
    let total_path_cmds = count_path_binaries();
    let total_services = count_systemd_services();
    let (commands, packages, services) = store.count_by_type();
    let total_known = store.total_objects();

    // Calculate progress
    let total_possible = total_path_cmds + total_services;
    let progress = if total_possible > 0 {
        (total_known as f64 / total_possible as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Determine status
    let status = if progress >= 99.0 {
        "Completed".green().to_string()
    } else if progress > 0.0 {
        "Scanning".yellow().to_string()
    } else {
        "Waiting".to_string()
    };

    println!("  Status:       {}", status);
    println!(
        "  Commands:     {}/{} ({})",
        commands,
        total_path_cmds,
        format_percent((commands as f64 / total_path_cmds.max(1) as f64) * 100.0)
    );
    println!("  Packages:     {}", packages);
    println!(
        "  Services:     {}/{} ({})",
        services,
        total_services,
        format_percent((services as f64 / total_services.max(1) as f64) * 100.0)
    );
    println!("  Progress:     {}", format_percent(progress));

    // ETA calculation
    let eta = if progress < 99.0 && progress > 0.0 {
        // Rough estimate based on typical scan rate
        let remaining = (100.0 - progress) / 100.0 * total_possible as f64;
        let rate = 50.0; // Assume ~50 items/sec
        let secs = (remaining / rate) as u64;
        if secs < 60 {
            format!("~{}s", secs)
        } else {
            format!("~{}m", secs / 60)
        }
    } else {
        "n/a".to_string()
    };
    println!("  ETA:          {}", eta);

    println!();
}

fn print_health_section(
    error_index: &ErrorIndex,
    service_index: &ServiceIndex,
    log_scan_state: &LogScanState,
) {
    println!("{}", "[HEALTH]".cyan());

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

    // Get intrusion count
    let intrusion_index = IntrusionIndex::load();
    let intrusions = intrusion_index.recent_high_severity(86400, 1).len();

    // Get failed services count
    let failed_services = service_index.failed_count;

    println!("  Errors (24h):     {}", errors_24h);
    println!("  Warnings (24h):   {}", warnings_24h);
    println!("  Intrusions:       {} detected", intrusions);
    println!("  Failed services:  {}", failed_services);

    // Log scanner status
    let scanner_status = if log_scan_state.running {
        "running".green().to_string()
    } else {
        "idle".to_string()
    };
    let last_scan = format_time_ago(log_scan_state.last_scan_at);
    println!(
        "  Log scanner:      {} (last scan {}, +{} errors, +{} warnings)",
        scanner_status, last_scan, log_scan_state.new_errors, log_scan_state.new_warnings
    );

    println!();
}

// ============================================================================
// Helper Functions
// ============================================================================

struct DaemonStatus {
    running: bool,
}

async fn check_daemon_status() -> DaemonStatus {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    let result = client
        .get("http://127.0.0.1:7865/v1/health")
        .send()
        .await;

    DaemonStatus {
        running: result.is_ok(),
    }
}

async fn check_ollama_status() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .is_ok()
}

fn get_daemon_uptime() -> String {
    // Try to get uptime from systemd
    let output = std::process::Command::new("systemctl")
        .args(["show", "annad", "--property=ActiveEnterTimestamp"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(ts_str) = stdout.strip_prefix("ActiveEnterTimestamp=") {
            let ts_str = ts_str.trim();
            if !ts_str.is_empty() && ts_str != "n/a" {
                // Parse timestamp and calculate uptime
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

fn get_last_update_check() -> String {
    // Try to read from update state file
    let state_path = "/var/lib/anna/update_state.json";
    if let Ok(content) = fs::read_to_string(state_path) {
        if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(ts) = state.get("last_check_at").and_then(|v| v.as_u64()) {
                let result = if let Some(success) = state.get("last_success").and_then(|v| v.as_bool()) {
                    if success { "ok" } else { "failed" }
                } else {
                    "ok"
                };
                return format!("{} ({})", format_time_ago(ts), result);
            }
        }
    }

    "never".to_string()
}
