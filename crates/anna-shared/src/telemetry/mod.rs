//! Telemetry modules for system state snapshots.
//!
//! Provides measured (not imagined) system deltas for REPL greeting.
//!
//! v0.0.29: Initial implementation with boot time and package change tracking.

pub mod boot;
pub mod pacman;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Combined telemetry snapshot for REPL greeting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    /// Boot time delta in milliseconds (current - previous)
    pub boot_delta_ms: Option<i64>,
    /// Number of package changes since last check
    pub package_changes: Option<usize>,
    /// Timestamp when snapshot was captured (epoch seconds)
    pub captured_at_ts: u64,
}

impl TelemetrySnapshot {
    /// Create a new snapshot with current timestamp
    pub fn new() -> Self {
        let captured_at_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            captured_at_ts,
            ..Default::default()
        }
    }

    /// Collect telemetry from boot and pacman modules
    pub fn collect() -> Self {
        let mut snap = Self::new();

        // Boot time delta
        if let Some(boot_snap) = boot::load_boot_snapshot() {
            snap.boot_delta_ms = boot_snap.delta_ms();
        }

        // Package changes
        if let Some(pacman_snap) = pacman::load_pacman_snapshot() {
            snap.package_changes = Some(pacman_snap.recent_events.len());
        }

        snap
    }

    /// Check if any telemetry data is available
    pub fn has_data(&self) -> bool {
        self.boot_delta_ms.is_some() || self.package_changes.is_some()
    }
}

/// Get the telemetry directory path
pub fn telemetry_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".anna").join("telemetry")
}

/// Clear all telemetry data (for reset)
pub fn clear_telemetry() -> std::io::Result<()> {
    let dir = telemetry_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_snapshot_new() {
        let snap = TelemetrySnapshot::new();
        assert!(snap.captured_at_ts > 0);
        assert!(snap.boot_delta_ms.is_none());
        assert!(snap.package_changes.is_none());
    }

    #[test]
    fn test_telemetry_snapshot_has_data() {
        let mut snap = TelemetrySnapshot::new();
        assert!(!snap.has_data());

        snap.boot_delta_ms = Some(-500);
        assert!(snap.has_data());
    }
}
