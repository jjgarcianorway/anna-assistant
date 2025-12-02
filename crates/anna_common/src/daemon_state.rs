//! Daemon State v7.38.0 - Crash Logging and Status Snapshots
//!
//! Provides:
//! - last_start.json: Written on every daemon start attempt
//! - last_crash.json: Written on panic/fatal error
//! - status_snapshot.json: Written every 60s with daemon status
//!
//! annactl status reads these files only - no live probing.

use crate::config::DATA_DIR;
use crate::atomic_write;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Internal state directory
pub const INTERNAL_DIR: &str = "/var/lib/anna/internal";

/// Last start attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastStart {
    /// When the start was attempted
    pub started_at: DateTime<Utc>,
    /// Version of the daemon
    pub version: String,
    /// PID of the daemon
    pub pid: u32,
    /// Whether startup completed successfully
    pub startup_complete: bool,
    /// Binary path
    pub binary_path: Option<String>,
}

impl LastStart {
    pub fn file_path() -> PathBuf {
        PathBuf::from(INTERNAL_DIR).join("last_start.json")
    }

    pub fn write_start() -> std::io::Result<()> {
        let start = LastStart {
            started_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            pid: std::process::id(),
            startup_complete: false,
            binary_path: std::env::current_exe().ok().map(|p| p.to_string_lossy().to_string()),
        };
        start.save()
    }

    pub fn mark_startup_complete() -> std::io::Result<()> {
        if let Some(mut start) = Self::load() {
            start.startup_complete = true;
            start.save()
        } else {
            Ok(())
        }
    }

    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        std::fs::read_to_string(&path).ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(INTERNAL_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::file_path().to_string_lossy(), &content)
    }
}

/// Last crash record - written on panic or fatal error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastCrash {
    /// When the crash occurred
    pub crashed_at: DateTime<Utc>,
    /// Version that crashed
    pub version: String,
    /// Reason for the crash
    pub reason: String,
    /// Component that failed (config, db, permissions, etc.)
    pub component: Option<String>,
    /// Backtrace (truncated)
    pub backtrace: Option<String>,
    /// System error code if applicable
    pub errno: Option<i32>,
}

impl LastCrash {
    pub fn file_path() -> PathBuf {
        PathBuf::from(INTERNAL_DIR).join("last_crash.json")
    }

    pub fn write(reason: &str, component: Option<&str>, backtrace: Option<&str>) {
        let crash = LastCrash {
            crashed_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            reason: reason.to_string(),
            component: component.map(|s| s.to_string()),
            backtrace: backtrace.map(|s| {
                // Truncate backtrace to reasonable size
                if s.len() > 4096 {
                    format!("{}...(truncated)", &s[..4096])
                } else {
                    s.to_string()
                }
            }),
            errno: None,
        };
        let _ = crash.save();
    }

    pub fn write_with_errno(reason: &str, component: Option<&str>, errno: i32) {
        let crash = LastCrash {
            crashed_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            reason: reason.to_string(),
            component: component.map(|s| s.to_string()),
            backtrace: None,
            errno: Some(errno),
        };
        let _ = crash.save();
    }

    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        std::fs::read_to_string(&path).ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(INTERNAL_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::file_path().to_string_lossy(), &content)
    }

    /// Format for status display
    pub fn format_summary(&self) -> String {
        let time = self.crashed_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let component = self.component.as_deref().unwrap_or("unknown");
        format!("{} {} ({})", time, self.reason, component)
    }
}

/// Status snapshot - written by daemon, read by annactl status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatusSnapshot {
    /// When this snapshot was taken
    pub snapshot_at: Option<DateTime<Utc>>,
    /// Daemon version
    pub version: String,
    /// Daemon PID
    pub pid: u32,
    /// Daemon uptime in seconds
    pub uptime_secs: u64,
    /// Whether daemon is healthy
    pub healthy: bool,
    /// Total objects in knowledge store
    pub knowledge_objects: usize,
    /// Telemetry sample count (24h)
    pub telemetry_samples_24h: u64,
    /// Last inventory scan time
    pub last_scan_at: Option<DateTime<Utc>>,
    /// Last scan duration in ms
    pub last_scan_duration_ms: u64,
    /// CPU alert if any
    pub cpu_alert: Option<String>,
    /// Memory usage MB
    pub memory_mb: u64,
    /// Disk usage info
    pub disk_info: Option<String>,
    /// Network IO summary
    pub network_io: Option<String>,
    /// Update check state
    pub update_last_check: Option<DateTime<Utc>>,
    pub update_next_check: Option<DateTime<Utc>>,
    pub update_result: Option<String>,
    /// Instrumentation installed count
    pub instrumentation_count: usize,
    /// Active alerts count
    pub alerts_critical: usize,
    pub alerts_warning: usize,
}

impl StatusSnapshot {
    pub fn file_path() -> PathBuf {
        PathBuf::from(INTERNAL_DIR).join("status_snapshot.json")
    }

    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        std::fs::read_to_string(&path).ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(INTERNAL_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::file_path().to_string_lossy(), &content)
    }

    /// Get snapshot age in seconds
    pub fn age_secs(&self) -> u64 {
        self.snapshot_at
            .map(|t| (Utc::now() - t).num_seconds().max(0) as u64)
            .unwrap_or(0)
    }

    /// Format age for display
    pub fn format_age(&self) -> String {
        let secs = self.age_secs();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h {}m ago", secs / 3600, (secs % 3600) / 60)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }

    /// Check if snapshot is stale (> 5 minutes)
    pub fn is_stale(&self) -> bool {
        self.age_secs() > 300
    }
}

/// Verify daemon can write to required directories
/// Returns Ok(()) if all directories are writable, or Err with details
pub fn verify_writable_dirs() -> Result<(), String> {
    let dirs = [
        "/var/lib/anna",
        "/var/lib/anna/internal",
        "/var/lib/anna/telemetry",
        "/var/lib/anna/knowledge",
        "/var/lib/anna/kdb",
    ];

    for dir in &dirs {
        // Try to create if doesn't exist
        if let Err(e) = std::fs::create_dir_all(dir) {
            return Err(format!("Cannot create {}: {}", dir, e));
        }

        // Try to write a test file
        let test_file = format!("{}/.write_test", dir);
        if let Err(e) = std::fs::write(&test_file, "test") {
            return Err(format!("Cannot write to {}: {}", dir, e));
        }
        let _ = std::fs::remove_file(&test_file);
    }

    Ok(())
}

/// Set appropriate permissions on internal files (0644 for readability)
pub fn set_file_permissions(path: &str) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o644);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_crash_format() {
        let crash = LastCrash {
            crashed_at: Utc::now(),
            version: "7.38.0".to_string(),
            reason: "Test error".to_string(),
            component: Some("db".to_string()),
            backtrace: None,
            errno: None,
        };
        let summary = crash.format_summary();
        assert!(summary.contains("Test error"));
        assert!(summary.contains("db"));
    }

    #[test]
    fn test_snapshot_age() {
        let snapshot = StatusSnapshot {
            snapshot_at: Some(Utc::now()),
            ..Default::default()
        };
        assert!(snapshot.age_secs() < 2);
        assert!(!snapshot.is_stale());
    }
}
