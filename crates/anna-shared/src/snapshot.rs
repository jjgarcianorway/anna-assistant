//! System snapshot for "what changed since last time" detection (v0.0.36).
//!
//! Captures minimal system state for delta detection without spamming users.
//! Only surfaces actionable changes that cross meaningful thresholds.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Thresholds for delta detection (anti-spam)
pub const DISK_WARN_THRESHOLD: u8 = 85;
pub const DISK_CRITICAL_THRESHOLD: u8 = 95;
pub const DISK_CHANGE_THRESHOLD: u8 = 5;
pub const MEMORY_HIGH_THRESHOLD: u8 = 85;
pub const MEMORY_CHANGE_THRESHOLD: u8 = 10;

/// System snapshot - minimal deterministic state capture
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Disk usage per mount point (mount -> percent_used)
    pub disk: BTreeMap<String, u8>,
    /// Failed systemd services (sorted)
    pub failed_services: Vec<String>,
    /// Memory total bytes
    pub memory_total_bytes: u64,
    /// Memory used bytes
    pub memory_used_bytes: u64,
    /// Capture timestamp (unix seconds, internal use only)
    #[serde(default)]
    pub captured_at: u64,
}

impl SystemSnapshot {
    /// Create empty snapshot
    pub fn new() -> Self {
        Self::default()
    }

    /// Create snapshot with current timestamp
    pub fn now() -> Self {
        Self {
            captured_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            ..Default::default()
        }
    }

    /// Add disk usage for a mount point
    pub fn add_disk(&mut self, mount: &str, percent: u8) {
        self.disk.insert(mount.to_string(), percent);
    }

    /// Add a failed service
    pub fn add_failed_service(&mut self, unit: &str) {
        if !self.failed_services.contains(&unit.to_string()) {
            self.failed_services.push(unit.to_string());
            self.failed_services.sort();
        }
    }

    /// Set memory stats
    pub fn set_memory(&mut self, total: u64, used: u64) {
        self.memory_total_bytes = total;
        self.memory_used_bytes = used;
    }

    /// Get memory usage percent
    pub fn memory_percent(&self) -> u8 {
        if self.memory_total_bytes == 0 {
            0
        } else {
            ((self.memory_used_bytes as f64 / self.memory_total_bytes as f64) * 100.0) as u8
        }
    }

    /// Check if snapshot has any data
    pub fn is_empty(&self) -> bool {
        self.disk.is_empty() && self.failed_services.is_empty() && self.memory_total_bytes == 0
    }

    /// Get age in seconds (0 if no timestamp)
    pub fn age_seconds(&self) -> u64 {
        if self.captured_at == 0 {
            return u64::MAX;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.captured_at)
    }

    /// Check if snapshot is fresh (within max_age_seconds)
    pub fn is_fresh(&self, max_age_seconds: u64) -> bool {
        self.age_seconds() <= max_age_seconds
    }
}

/// A single delta item between snapshots
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaItem {
    /// Disk usage crossed warning threshold
    DiskWarning { mount: String, prev: u8, curr: u8 },
    /// Disk usage crossed critical threshold
    DiskCritical { mount: String, prev: u8, curr: u8 },
    /// Disk usage increased significantly
    DiskIncreased { mount: String, prev: u8, curr: u8 },
    /// New failed service appeared
    NewFailedService { unit: String },
    /// Service recovered (was failed, now ok)
    ServiceRecovered { unit: String },
    /// Memory crossed high threshold
    MemoryHigh { prev_percent: u8, curr_percent: u8 },
    /// Memory increased significantly
    MemoryIncreased { prev_percent: u8, curr_percent: u8 },
}

impl DeltaItem {
    /// Format as single line for display
    pub fn format(&self) -> String {
        match self {
            Self::DiskWarning { mount, prev, curr } => {
                format!("⚠ Disk {} at {}% (was {}%)", mount, curr, prev)
            }
            Self::DiskCritical { mount, prev, curr } => {
                format!("🔴 Disk {} CRITICAL at {}% (was {}%)", mount, curr, prev)
            }
            Self::DiskIncreased { mount, prev, curr } => {
                format!("📈 Disk {} increased to {}% (was {}%)", mount, curr, prev)
            }
            Self::NewFailedService { unit } => {
                format!("🔴 Service {} failed", unit)
            }
            Self::ServiceRecovered { unit } => {
                format!("✅ Service {} recovered", unit)
            }
            Self::MemoryHigh { prev_percent, curr_percent } => {
                format!("⚠ Memory high at {}% (was {}%)", curr_percent, prev_percent)
            }
            Self::MemoryIncreased { prev_percent, curr_percent } => {
                format!("📈 Memory increased to {}% (was {}%)", curr_percent, prev_percent)
            }
        }
    }

    /// Check if this is an error-level delta
    pub fn is_error(&self) -> bool {
        matches!(self, Self::DiskCritical { .. } | Self::NewFailedService { .. })
    }

    /// Check if this is a warning-level delta
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::DiskWarning { .. } | Self::MemoryHigh { .. })
    }
}

/// Compare two snapshots and return meaningful deltas only
pub fn diff_snapshots(prev: &SystemSnapshot, curr: &SystemSnapshot) -> Vec<DeltaItem> {
    let mut deltas = Vec::new();

    // Disk deltas (deterministic order via BTreeMap)
    for (mount, &curr_pct) in &curr.disk {
        let prev_pct = prev.disk.get(mount).copied().unwrap_or(0);

        // Check critical threshold crossing
        if curr_pct >= DISK_CRITICAL_THRESHOLD && prev_pct < DISK_CRITICAL_THRESHOLD {
            deltas.push(DeltaItem::DiskCritical {
                mount: mount.clone(),
                prev: prev_pct,
                curr: curr_pct,
            });
        }
        // Check warning threshold crossing
        else if curr_pct >= DISK_WARN_THRESHOLD && prev_pct < DISK_WARN_THRESHOLD {
            deltas.push(DeltaItem::DiskWarning {
                mount: mount.clone(),
                prev: prev_pct,
                curr: curr_pct,
            });
        }
        // Check significant increase
        else if curr_pct >= prev_pct + DISK_CHANGE_THRESHOLD {
            deltas.push(DeltaItem::DiskIncreased {
                mount: mount.clone(),
                prev: prev_pct,
                curr: curr_pct,
            });
        }
    }

    // Failed services deltas
    for unit in &curr.failed_services {
        if !prev.failed_services.contains(unit) {
            deltas.push(DeltaItem::NewFailedService { unit: unit.clone() });
        }
    }
    for unit in &prev.failed_services {
        if !curr.failed_services.contains(unit) {
            deltas.push(DeltaItem::ServiceRecovered { unit: unit.clone() });
        }
    }

    // Memory deltas
    let prev_mem = prev.memory_percent();
    let curr_mem = curr.memory_percent();

    if curr_mem >= MEMORY_HIGH_THRESHOLD && prev_mem < MEMORY_HIGH_THRESHOLD {
        deltas.push(DeltaItem::MemoryHigh {
            prev_percent: prev_mem,
            curr_percent: curr_mem,
        });
    } else if curr_mem >= prev_mem + MEMORY_CHANGE_THRESHOLD {
        deltas.push(DeltaItem::MemoryIncreased {
            prev_percent: prev_mem,
            curr_percent: curr_mem,
        });
    }

    deltas
}

/// Format deltas as text for display (deterministic, no walls of text)
pub fn format_deltas_text(deltas: &[DeltaItem]) -> String {
    if deltas.is_empty() {
        return "No new warnings since last check.".to_string();
    }

    let mut lines: Vec<String> = deltas.iter().map(|d| d.format()).collect();

    // Cap at 5 lines to avoid spam
    if lines.len() > 5 {
        let omitted = lines.len() - 4;
        lines.truncate(4);
        lines.push(format!("... and {} more changes", omitted));
    }

    lines.join("\n")
}

/// Check if deltas contain any errors or warnings worth showing
pub fn has_actionable_deltas(deltas: &[DeltaItem]) -> bool {
    deltas.iter().any(|d| d.is_error() || d.is_warning())
}

// === Snapshot capture from probe results ===

use crate::rpc::ProbeResult;

/// Capture snapshot from probe results
pub fn capture_snapshot(probes: &[ProbeResult]) -> SystemSnapshot {
    let mut snapshot = SystemSnapshot::now();

    for probe in probes {
        if probe.exit_code != 0 {
            continue; // Skip failed probes
        }

        // Parse df output for disk usage
        if probe.command.contains("df") {
            parse_df_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // Parse free output for memory
        if probe.command.contains("free") {
            parse_free_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // Parse systemctl --failed for failed services
        if probe.command.contains("--failed") {
            parse_failed_services_into_snapshot(&probe.stdout, &mut snapshot);
        }
    }

    snapshot
}

/// Parse df -h output into snapshot
fn parse_df_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // df output format: Filesystem Size Used Avail Use% Mounted
    for line in output.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let use_percent = parts[4].trim_end_matches('%');
            let mount = parts[5];

            // Only track relevant mounts
            if mount == "/" || mount == "/home" || mount == "/var" || mount == "/tmp"
                || mount.starts_with("/mnt") || mount.starts_with("/media")
            {
                if let Ok(pct) = use_percent.parse::<u8>() {
                    snapshot.add_disk(mount, pct);
                }
            }
        }
    }
}

/// Parse free -b output into snapshot
fn parse_free_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // Try to find "Mem:" line
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // Format: Mem: total used free...
                if let (Ok(total), Ok(used)) = (
                    parts[1].parse::<u64>(),
                    parts[2].parse::<u64>(),
                ) {
                    snapshot.set_memory(total, used);
                    return;
                }
            }
        }
    }
}

/// Parse systemctl --failed output into snapshot
fn parse_failed_services_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // systemctl --failed output has units that are in failed state
    // Format: [●] UNIT LOAD ACTIVE SUB DESCRIPTION
    // The bullet point (●) may appear before the unit name
    for line in output.lines() {
        let line = line.trim();
        // Skip header and summary lines
        if line.is_empty()
            || line.starts_with("UNIT")
            || line.starts_with("LOAD")
            || line.contains("loaded units listed")
            || line.contains("0 loaded units")
        {
            continue;
        }

        // Extract unit name - handle bullet point prefix (●)
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            // Skip the bullet point and empty strings
            if part == "●" || part.is_empty() {
                continue;
            }
            // Found the unit name
            if part.ends_with(".service") || part.ends_with(".socket") || part.ends_with(".timer") {
                snapshot.add_failed_service(part);
                break; // Only take the first matching unit per line
            }
        }
    }
}

// === Persistence ===

/// Get snapshots directory path
pub fn snapshots_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("snapshots")
}

/// Get last snapshot file path
pub fn last_snapshot_path() -> PathBuf {
    snapshots_dir().join("last.json")
}

/// Load the last saved snapshot (if any)
pub fn load_last_snapshot() -> Option<SystemSnapshot> {
    let path = last_snapshot_path();
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save snapshot as the "last" snapshot
pub fn save_snapshot(snapshot: &SystemSnapshot) -> std::io::Result<()> {
    let dir = snapshots_dir();
    std::fs::create_dir_all(&dir)?;
    let path = last_snapshot_path();
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Clear all snapshots (for reset)
pub fn clear_snapshots() -> std::io::Result<()> {
    let dir = snapshots_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

// Tests moved to tests/snapshot_tests.rs
