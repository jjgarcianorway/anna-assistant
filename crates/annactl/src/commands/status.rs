//! Status Command v5.7.1 - Clean Status Display
//!
//! Shows Anna daemon health and system coverage.
//!
//! Sections:
//! - [VERSION] Single version (annactl=annad)
//! - [DAEMON] Daemon state and uptime
//! - [INVENTORY] Meaningful counts
//! - [HEALTH] Log pipeline summary (24h) - only if issues found
//! - [SCANNER] Scanner operational status

use anyhow::Result;
use owo_colors::OwoColorize;
use std::os::unix::fs::MetadataExt;
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

    // [VERSION] - single line, they're always the same
    print_version_section().await;

    // [DAEMON]
    print_daemon_section().await;

    // [INVENTORY]
    print_inventory_section(&store, &inventory_progress);

    // [HEALTH] - only if issues found
    print_health_section(&error_index, &service_index);

    // [SCANNER]
    print_scanner_section(&log_scan_state);

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

async fn print_version_section() {
    println!("{}", "[VERSION]".cyan());

    // v5.7.1: Single version - annactl and annad are always the same
    println!("  Anna:       v{}", VERSION);
    println!();
}

async fn print_daemon_section() {
    println!("{}", "[DAEMON]".cyan());

    // Check daemon status via health endpoint and get uptime from it
    match get_daemon_info().await {
        Some(info) => {
            let uptime_str = format_duration_secs(info.uptime_secs);
            println!("  Status:     {} (up {})", "running".green(), uptime_str);
            println!("  Objects:    {}", info.objects_tracked);
        }
        None => {
            println!("  Status:     {}", "stopped".red());
        }
    }

    println!();
}

fn print_inventory_section(store: &KnowledgeStore, progress: &InventoryProgress) {
    println!("{}", "[INVENTORY]".cyan());

    let total_path_cmds = count_path_binaries();
    let total_services = count_systemd_services();
    let (commands, packages, services) = store.count_by_type();

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

    // Show scan status
    let has_data = commands > 0 || packages > 0 || services > 0;

    match progress.phase {
        InventoryPhase::Complete => {
            println!("  Status:     {}", "complete".green());
        }
        InventoryPhase::Idle if progress.initial_scan_complete || has_data => {
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
) {
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

    let intrusion_index = IntrusionIndex::load();
    let intrusions = intrusion_index.recent_high_severity(86400, 1).len();
    let failed_services = service_index.failed_count;

    // Only show if there's something to report
    let has_issues = errors_24h > 0 || warnings_24h > 0 || intrusions > 0 || failed_services > 0;

    if has_issues {
        println!("{}", "[HEALTH]".cyan());
        println!("  {}", "(last 24 hours)".dimmed());

        if errors_24h > 0 {
            println!("  Errors:     {}", errors_24h.to_string().red());
        }
        if warnings_24h > 0 {
            println!("  Warnings:   {}", warnings_24h.to_string().yellow());
        }
        if intrusions > 0 {
            println!("  Intrusions: {}", intrusions.to_string().red().bold());
        }
        if failed_services > 0 {
            println!("  Failed:     {}", failed_services.to_string().red());
        }

        println!();
    }
}

fn print_scanner_section(log_scan_state: &LogScanState) {
    println!("{}", "[SCANNER]".cyan());
    let scanner_status = if log_scan_state.running {
        "running".green().to_string()
    } else {
        "idle".to_string()
    };
    let last_scan = format_time_ago(log_scan_state.last_scan_at);
    println!("  Status:     {} (last {})", scanner_status, last_scan);
    println!();
}

// ============================================================================
// Helper Functions
// ============================================================================

#[derive(serde::Deserialize)]
struct HealthResponse {
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    phase: String,
    uptime_secs: u64,
    objects_tracked: usize,
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

/// Get file permissions in Linux format (e.g., "root:root 644")
#[allow(dead_code)]
fn get_file_permissions(path: &str) -> String {
    let p = Path::new(path);
    if !p.exists() {
        return "---".to_string();
    }

    match std::fs::metadata(p) {
        Ok(meta) => {
            let mode = meta.mode() & 0o777;
            let uid = meta.uid();
            let gid = meta.gid();

            // Get user/group names
            let user = get_username(uid).unwrap_or_else(|| uid.to_string());
            let group = get_groupname(gid).unwrap_or_else(|| gid.to_string());

            format!("{}:{} {:o}", user, group, mode)
        }
        Err(_) => "---".to_string(),
    }
}

fn get_username(uid: u32) -> Option<String> {
    // Simple lookup via /etc/passwd parsing would be complex
    // For now, just return root for uid 0
    if uid == 0 {
        Some("root".to_string())
    } else {
        None
    }
}

fn get_groupname(gid: u32) -> Option<String> {
    if gid == 0 {
        Some("root".to_string())
    } else {
        None
    }
}
