//! Welcome summary and startup context awareness (Beta.209)
//!
//! This module provides deterministic welcome reports with zero LLM usage.
//! It tracks session metadata and generates system state summaries based on
//! telemetry diffs between the last run and current state.
//!
//! Philosophy:
//! - Zero LLM usage for welcome summaries
//! - Deterministic diff computation from telemetry
//! - Atomic file operations for state persistence
//! - Canonical [SUMMARY]/[DETAILS] format

use anyhow::{Context, Result};
use anna_common::telemetry::SystemTelemetry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Session metadata stored in /var/lib/anna/state/last_session.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Timestamp of last Anna invocation
    pub last_run: DateTime<Utc>,

    /// System snapshot at last run
    pub last_telemetry: TelemetrySnapshot,

    /// Anna version at last run
    pub anna_version: String,
}

/// Minimal telemetry snapshot for diff computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    /// CPU count
    pub cpu_count: usize,

    /// Total RAM in MB
    pub total_ram_mb: u64,

    /// Hostname
    pub hostname: String,

    /// Kernel version
    pub kernel_version: String,

    /// Installed packages count
    pub package_count: usize,

    /// Available disk space in GB
    pub available_disk_gb: u64,
}

/// Get path to session metadata file
fn get_session_metadata_path() -> PathBuf {
    PathBuf::from("/var/lib/anna/state/last_session.json")
}

/// Load last session metadata from disk
///
/// Returns None if file doesn't exist or is invalid (first run scenario)
pub fn load_last_session() -> Result<Option<SessionMetadata>> {
    let path = get_session_metadata_path();

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .context("Failed to read last session metadata")?;

    let metadata: SessionMetadata = serde_json::from_str(&content)
        .context("Failed to parse last session metadata")?;

    Ok(Some(metadata))
}

/// Save current session metadata to disk
///
/// Uses atomic write (write to temp file, then rename) for safety
pub fn save_session_metadata(telemetry: TelemetrySnapshot) -> Result<()> {
    let path = get_session_metadata_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create session state directory")?;
    }

    let metadata = SessionMetadata {
        last_run: Utc::now(),
        last_telemetry: telemetry,
        anna_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(&metadata)
        .context("Failed to serialize session metadata")?;

    fs::write(&temp_path, content)
        .context("Failed to write temp session metadata")?;

    fs::rename(&temp_path, &path)
        .context("Failed to move temp session metadata to final location")?;

    Ok(())
}

/// Generate welcome report comparing last session to current state
///
/// Returns a canonical [SUMMARY]/[DETAILS] formatted report.
/// Zero LLM usage - purely deterministic based on telemetry diff.
pub fn generate_welcome_report(
    last_session: Option<SessionMetadata>,
    current_telemetry: TelemetrySnapshot,
) -> String {
    match last_session {
        None => generate_first_run_welcome(&current_telemetry),
        Some(last) => generate_returning_user_welcome(&last, &current_telemetry),
    }
}

/// Generate welcome for first-time run (no previous session)
fn generate_first_run_welcome(current: &TelemetrySnapshot) -> String {
    format!(
        r#"[SUMMARY]
Welcome to Anna Assistant! This is your first session.

[DETAILS]
System Information:
- Hostname: {}
- Kernel: {}
- CPU Cores: {}
- RAM: {} GB
- Disk Space: {} GB available
- Packages: {} installed

Anna is ready to help with Arch Linux system management.
Type your questions in natural language or use 'status' for system health."#,
        current.hostname,
        current.kernel_version,
        current.cpu_count,
        current.total_ram_mb / 1024,
        current.available_disk_gb,
        current.package_count,
    )
}

/// Generate welcome for returning user (diff from last session)
fn generate_returning_user_welcome(
    last: &SessionMetadata,
    current: &TelemetrySnapshot,
) -> String {
    let time_since = Utc::now() - last.last_run;
    let changes = compute_system_changes(&last.last_telemetry, current);

    let summary = if changes.is_empty() {
        format!(
            "Welcome back! No system changes detected since last run ({}).",
            format_time_since(time_since.num_seconds())
        )
    } else {
        format!(
            "Welcome back! {} system change{} detected since last run ({}).",
            changes.len(),
            if changes.len() == 1 { "" } else { "s" },
            format_time_since(time_since.num_seconds())
        )
    };

    let details = if changes.is_empty() {
        format!(
            r#"System Status:
- Hostname: {}
- Kernel: {}
- CPU Cores: {}
- RAM: {} GB
- Disk Space: {} GB available
- Packages: {} installed"#,
            current.hostname,
            current.kernel_version,
            current.cpu_count,
            current.total_ram_mb / 1024,
            current.available_disk_gb,
            current.package_count,
        )
    } else {
        let change_list = changes.join("\n");
        format!(
            r#"Recent Changes:
{}

Current System:
- Hostname: {}
- Kernel: {}
- CPU Cores: {}
- RAM: {} GB
- Disk Space: {} GB available
- Packages: {} installed"#,
            change_list,
            current.hostname,
            current.kernel_version,
            current.cpu_count,
            current.total_ram_mb / 1024,
            current.available_disk_gb,
            current.package_count,
        )
    };

    format!("[SUMMARY]\n{}\n\n[DETAILS]\n{}", summary, details)
}

/// Compute deterministic system changes between two telemetry snapshots
fn compute_system_changes(last: &TelemetrySnapshot, current: &TelemetrySnapshot) -> Vec<String> {
    let mut changes = Vec::new();

    // Hostname change
    if last.hostname != current.hostname {
        changes.push(format!(
            "- Hostname changed: {} → {}",
            last.hostname, current.hostname
        ));
    }

    // Kernel update
    if last.kernel_version != current.kernel_version {
        changes.push(format!(
            "- Kernel updated: {} → {}",
            last.kernel_version, current.kernel_version
        ));
    }

    // CPU count change (rare, but possible with virtualization)
    if last.cpu_count != current.cpu_count {
        changes.push(format!(
            "- CPU cores changed: {} → {}",
            last.cpu_count, current.cpu_count
        ));
    }

    // RAM change
    if last.total_ram_mb != current.total_ram_mb {
        changes.push(format!(
            "- RAM changed: {} GB → {} GB",
            last.total_ram_mb / 1024,
            current.total_ram_mb / 1024
        ));
    }

    // Package changes
    let pkg_diff = current.package_count as i64 - last.package_count as i64;
    if pkg_diff != 0 {
        if pkg_diff > 0 {
            changes.push(format!("- Packages: +{} installed", pkg_diff));
        } else {
            changes.push(format!("- Packages: {} removed", pkg_diff.abs()));
        }
    }

    // Disk space change (only report significant changes > 1 GB)
    let disk_diff = current.available_disk_gb as i64 - last.available_disk_gb as i64;
    if disk_diff.abs() > 1 {
        if disk_diff > 0 {
            changes.push(format!("- Disk space: +{} GB available", disk_diff));
        } else {
            changes.push(format!("- Disk space: {} GB used", disk_diff.abs()));
        }
    }

    changes
}

/// Format time duration in human-readable form
fn format_time_since(seconds: i64) -> String {
    if seconds < 60 {
        "less than a minute ago".to_string()
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = seconds / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}

/// Create TelemetrySnapshot from SystemTelemetry
///
/// Extracts minimal fields needed for diff computation
pub fn create_telemetry_snapshot(telemetry: &SystemTelemetry) -> TelemetrySnapshot {
    // Get hostname
    let hostname = Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get kernel version
    let kernel_version = Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    TelemetrySnapshot {
        cpu_count: telemetry.cpu.cores as usize,
        total_ram_mb: telemetry.memory.total_mb,
        hostname,
        kernel_version,
        package_count: telemetry.packages.total_installed as usize,
        available_disk_gb: telemetry.disks.iter()
            .map(|d| {
                let available_mb = d.total_mb.saturating_sub(d.used_mb);
                (available_mb / 1024) as u64
            })
            .sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_run_welcome() {
        let telemetry = TelemetrySnapshot {
            cpu_count: 8,
            total_ram_mb: 16384,
            hostname: "archlinux".to_string(),
            kernel_version: "6.17.8-arch1-1".to_string(),
            package_count: 1234,
            available_disk_gb: 500,
        };

        let report = generate_first_run_welcome(&telemetry);

        assert!(report.contains("[SUMMARY]"));
        assert!(report.contains("[DETAILS]"));
        assert!(report.contains("first session"));
        assert!(report.contains("archlinux"));
        assert!(report.contains("6.17.8-arch1-1"));
        assert!(report.contains("8")); // CPU cores
        assert!(report.contains("16 GB")); // RAM
        assert!(report.contains("500 GB")); // Disk
        assert!(report.contains("1234")); // Packages
    }

    #[test]
    fn test_no_changes_welcome() {
        let telemetry = TelemetrySnapshot {
            cpu_count: 8,
            total_ram_mb: 16384,
            hostname: "archlinux".to_string(),
            kernel_version: "6.17.8-arch1-1".to_string(),
            package_count: 1234,
            available_disk_gb: 500,
        };

        let last_session = SessionMetadata {
            last_run: Utc::now() - chrono::Duration::hours(2),
            last_telemetry: telemetry.clone(),
            anna_version: "5.7.0-beta.209".to_string(),
        };

        let report = generate_returning_user_welcome(&last_session, &telemetry);

        assert!(report.contains("[SUMMARY]"));
        assert!(report.contains("[DETAILS]"));
        assert!(report.contains("Welcome back"));
        assert!(report.contains("No system changes detected"));
        assert!(report.contains("2 hours ago"));
    }

    #[test]
    fn test_package_changes_detected() {
        let last_telemetry = TelemetrySnapshot {
            cpu_count: 8,
            total_ram_mb: 16384,
            hostname: "archlinux".to_string(),
            kernel_version: "6.17.8-arch1-1".to_string(),
            package_count: 1234,
            available_disk_gb: 500,
        };

        let current_telemetry = TelemetrySnapshot {
            package_count: 1240, // 6 packages added
            ..last_telemetry.clone()
        };

        let changes = compute_system_changes(&last_telemetry, &current_telemetry);

        assert_eq!(changes.len(), 1);
        assert!(changes[0].contains("Packages: +6 installed"));
    }

    #[test]
    fn test_kernel_update_detected() {
        let last_telemetry = TelemetrySnapshot {
            cpu_count: 8,
            total_ram_mb: 16384,
            hostname: "archlinux".to_string(),
            kernel_version: "6.17.8-arch1-1".to_string(),
            package_count: 1234,
            available_disk_gb: 500,
        };

        let current_telemetry = TelemetrySnapshot {
            kernel_version: "6.18.0-arch1-1".to_string(),
            ..last_telemetry.clone()
        };

        let changes = compute_system_changes(&last_telemetry, &current_telemetry);

        assert_eq!(changes.len(), 1);
        assert!(changes[0].contains("Kernel updated"));
        assert!(changes[0].contains("6.17.8-arch1-1"));
        assert!(changes[0].contains("6.18.0-arch1-1"));
    }

    #[test]
    fn test_multiple_changes_detected() {
        let last_telemetry = TelemetrySnapshot {
            cpu_count: 8,
            total_ram_mb: 16384,
            hostname: "archlinux".to_string(),
            kernel_version: "6.17.8-arch1-1".to_string(),
            package_count: 1234,
            available_disk_gb: 500,
        };

        let current_telemetry = TelemetrySnapshot {
            kernel_version: "6.18.0-arch1-1".to_string(),
            package_count: 1240,
            available_disk_gb: 495, // 5 GB used
            ..last_telemetry.clone()
        };

        let changes = compute_system_changes(&last_telemetry, &current_telemetry);

        assert_eq!(changes.len(), 3); // kernel, packages, disk
        assert!(changes.iter().any(|c| c.contains("Kernel updated")));
        assert!(changes.iter().any(|c| c.contains("Packages: +6")));
        assert!(changes.iter().any(|c| c.contains("Disk space: 5 GB used")));
    }

    #[test]
    fn test_format_time_since() {
        assert_eq!(format_time_since(30), "less than a minute ago");
        assert_eq!(format_time_since(120), "2 minutes ago");
        assert_eq!(format_time_since(3600), "1 hour ago");
        assert_eq!(format_time_since(7200), "2 hours ago");
        assert_eq!(format_time_since(86400), "1 day ago");
        assert_eq!(format_time_since(172800), "2 days ago");
    }
}
