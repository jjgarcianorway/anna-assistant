//! Status Command v5.4.0 - Daemon Health
//!
//! Shows Anna daemon health and system coverage.
//!
//! Sections:
//! - [VERSION] annactl/annad versions
//! - [DAEMON] Daemon state and uptime
//! - [INVENTORY] Scan progress with ETA
//! - [HEALTH] Log pipeline summary (24h) - only if issues found

use anyhow::Result;
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    ErrorIndex, ServiceIndex, IntrusionIndex, LogScanState, InventoryProgress,
    KnowledgeStore, count_path_binaries, count_systemd_services,
    format_duration_secs, format_time_ago, format_percent, InventoryPhase,
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
    let inventory_progress = InventoryProgress::load();

    // [VERSION]
    print_version_section();

    // [DAEMON]
    print_daemon_section().await;

    // [INVENTORY] with progress and ETA
    print_inventory_section(&store, &inventory_progress);

    // [HEALTH] - only if issues found (signal not noise)
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

fn print_inventory_section(store: &KnowledgeStore, progress: &InventoryProgress) {
    println!("{}", "[INVENTORY]".cyan());

    // Get counts
    let total_path_cmds = count_path_binaries();
    let total_services = count_systemd_services();
    let (commands, packages, services) = store.count_by_type();

    // Calculate coverage percentages
    let cmd_pct = (commands as f64 / total_path_cmds.max(1) as f64) * 100.0;
    let svc_pct = (services as f64 / total_services.max(1) as f64) * 100.0;

    println!(
        "  Commands:   {}/{} ({})",
        commands,
        total_path_cmds,
        format_percent(cmd_pct)
    );
    println!("  Packages:   {}", packages);
    println!(
        "  Services:   {}/{} ({})",
        services,
        total_services,
        format_percent(svc_pct)
    );

    // v5.6.0: Fix inventory status - never show "waiting" if we have data
    // If we have any data (commands/packages/services > 0), the scan is complete
    let has_data = commands > 0 || packages > 0 || services > 0;

    // Show scan status with ETA if scanning
    match progress.phase {
        InventoryPhase::Complete => {
            println!("  Status:     {}", "complete".green());
        }
        InventoryPhase::Idle if progress.initial_scan_complete || has_data => {
            // v5.6.0: If we have data, we're complete even if flag wasn't set
            println!("  Status:     {}", "complete".green());
        }
        InventoryPhase::Idle => {
            println!("  Status:     waiting");
        }
        InventoryPhase::PriorityScan => {
            if let Some(target) = &progress.priority_target {
                println!("  Status:     {} ({})", "priority scan".yellow(), target);
            } else {
                println!("  Status:     {}", "priority scan".yellow());
            }
        }
        _ => {
            // Active scan - show progress and ETA
            let phase_name = progress.phase.as_str();
            let pct = progress.percent;
            let eta = progress.format_eta();
            println!(
                "  Status:     {} {}% (ETA: {})",
                phase_name.yellow(),
                pct,
                eta
            );
        }
    }

    println!();
}

fn print_health_section(
    error_index: &ErrorIndex,
    service_index: &ServiceIndex,
    log_scan_state: &LogScanState,
) {
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

    // Signal not noise: only show [HEALTH] section if there's something to report
    let has_issues = errors_24h > 0 || warnings_24h > 0 || intrusions > 0 || failed_services > 0;

    if has_issues {
        println!("{}", "[HEALTH]".cyan());
        println!("  {}", "(last 24 hours)".dimmed());

        if errors_24h > 0 {
            println!("  Errors:      {}", errors_24h.to_string().red());
        }
        if warnings_24h > 0 {
            println!("  Warnings:    {}", warnings_24h.to_string().yellow());
        }
        if intrusions > 0 {
            println!("  Intrusions:  {}", intrusions.to_string().red().bold());
        }
        if failed_services > 0 {
            println!("  Failed svcs: {}", failed_services.to_string().red());
        }

        println!();
    }

    // Always show scanner status (operational info)
    println!("{}", "[SCANNER]".cyan());
    let scanner_status = if log_scan_state.running {
        "running".green().to_string()
    } else {
        "idle".to_string()
    };
    let last_scan = format_time_ago(log_scan_state.last_scan_at);
    println!("  Status:      {} (last {})", scanner_status, last_scan);
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
                // v5.5.1: Parse systemctl timestamp format "Mon 2025-12-01 13:50:39 CET"
                // Use chrono's NaiveDateTime for simpler parsing
                let parts: Vec<&str> = ts_str.split_whitespace().collect();
                if parts.len() >= 3 {
                    // Skip day name, parse date and time
                    let date_time_str = format!("{} {}", parts[1], parts[2]);
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S") {
                        let start = dt.and_utc().timestamp() as u64;
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        // Account for timezone - assume local time is close enough
                        // For more accuracy we'd need to parse the timezone
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
