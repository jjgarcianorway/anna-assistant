//! Daemon State v7.42.0 - Crash Logging and Status Snapshots
//!
//! v7.42.0: Canonical paths for daemon/CLI contract
//! - All paths defined here are used by BOTH annad and annactl
//! - Status snapshot written immediately on startup + every 60s
//! - Schema versioning for forward compatibility
//!
//! Provides:
//! - last_start.json: Written on every daemon start attempt
//! - last_crash.json: Written on panic/fatal error
//! - status.json: Written immediately on startup and every 60s
//!
//! annactl reads these files + uses control socket for live check.

use crate::atomic_write;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// =============================================================================
// CANONICAL PATHS - used by BOTH annad and annactl
// =============================================================================

/// Base data directory
pub const DATA_DIR: &str = "/var/lib/anna";

/// Internal state directory
pub const INTERNAL_DIR: &str = "/var/lib/anna/internal";

/// Snapshots directory (daemon-written, CLI-read)
pub const SNAPSHOTS_DIR: &str = "/var/lib/anna/internal/snapshots";

/// Meta directory (delta detection)
pub const META_DIR: &str = "/var/lib/anna/internal/meta";

/// Status snapshot path (canonical)
pub const STATUS_SNAPSHOT_PATH: &str = "/var/lib/anna/internal/snapshots/status.json";

/// Current schema version for status snapshot
/// v2: Added telemetry, update state
/// v3: Added LLM bootstrap state (v0.0.5)
/// v4: Added helper tracking with provenance (v0.0.9)
pub const STATUS_SCHEMA_VERSION: u32 = 4;

/// Stale threshold in seconds (15 minutes)
pub const STALE_THRESHOLD_SECS: u64 = 900;

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
/// v7.42.0: Now at /var/lib/anna/internal/snapshots/status.json with schema versioning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatusSnapshot {
    /// Schema version for forward compatibility
    #[serde(default)]
    pub schema_version: u32,
    /// When this snapshot was taken (RFC3339)
    pub generated_at: Option<DateTime<Utc>>,
    /// Monotonic sequence number (for ordering)
    #[serde(default)]
    pub seq: u64,
    /// Daemon version
    pub version: String,
    /// Daemon PID
    pub pid: u32,
    /// Boot ID for this daemon instance
    pub boot_id: Option<String>,
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

    // v0.0.5: LLM bootstrap state
    /// Current bootstrap phase (detecting_ollama, installing_ollama, pulling_models, benchmarking, ready, error)
    #[serde(default)]
    pub llm_bootstrap_phase: Option<String>,
    /// Selected translator model (if ready)
    #[serde(default)]
    pub llm_translator_model: Option<String>,
    /// Selected junior model (if ready)
    #[serde(default)]
    pub llm_junior_model: Option<String>,
    /// Model being downloaded (if pulling)
    #[serde(default)]
    pub llm_downloading_model: Option<String>,
    /// Download progress percentage (0-100)
    #[serde(default)]
    pub llm_download_percent: Option<f64>,
    /// Download speed (bytes/sec)
    #[serde(default)]
    pub llm_download_speed: Option<f64>,
    /// Download ETA in seconds
    #[serde(default)]
    pub llm_download_eta_secs: Option<u64>,
    /// Last LLM error message
    #[serde(default)]
    pub llm_error: Option<String>,
    /// Hardware tier (low, medium, high)
    #[serde(default)]
    pub llm_hardware_tier: Option<String>,

    // v0.0.9: Helper tracking with provenance
    /// Helper summary (total, present, missing, anna-installed)
    #[serde(default)]
    pub helpers_total: usize,
    #[serde(default)]
    pub helpers_present: usize,
    #[serde(default)]
    pub helpers_missing: usize,
    #[serde(default)]
    pub helpers_anna_installed: usize,
    /// Detailed helper list (for status display)
    #[serde(default)]
    pub helpers: Vec<crate::helpers::HelperStatusEntry>,

    // v7.42.0: Legacy compatibility - map old field name (private, used only for deserialization)
    #[serde(alias = "snapshot_at", default, skip_serializing)]
    #[doc(hidden)]
    pub _snapshot_at_compat: Option<DateTime<Utc>>,
}

impl StatusSnapshot {
    /// Canonical file path (v7.42.0: moved to snapshots/)
    pub fn file_path() -> PathBuf {
        PathBuf::from(STATUS_SNAPSHOT_PATH)
    }

    /// Legacy file path (for migration)
    pub fn legacy_file_path() -> PathBuf {
        PathBuf::from(INTERNAL_DIR).join("status_snapshot.json")
    }

    /// Load snapshot - tries new path first, then legacy
    pub fn load() -> Option<Self> {
        // Try canonical path first
        if let Some(snapshot) = Self::load_from_path(&Self::file_path()) {
            return Some(snapshot);
        }

        // Try legacy path
        if let Some(mut snapshot) = Self::load_from_path(&Self::legacy_file_path()) {
            // Handle legacy snapshot_at field
            if snapshot.generated_at.is_none() && snapshot._snapshot_at_compat.is_some() {
                snapshot.generated_at = snapshot._snapshot_at_compat.take();
            }
            return Some(snapshot);
        }

        None
    }

    fn load_from_path(path: &PathBuf) -> Option<Self> {
        std::fs::read_to_string(path).ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    /// Save snapshot atomically
    pub fn save(&self) -> std::io::Result<()> {
        // Ensure directory exists
        std::fs::create_dir_all(SNAPSHOTS_DIR)?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Atomic write
        atomic_write(STATUS_SNAPSHOT_PATH, &content)?;

        // Set permissions (0644 for readability)
        set_file_permissions(STATUS_SNAPSHOT_PATH);

        Ok(())
    }

    /// Get snapshot age in seconds
    pub fn age_secs(&self) -> u64 {
        self.generated_at
            .map(|t| (Utc::now() - t).num_seconds().max(0) as u64)
            .unwrap_or(u64::MAX) // If no timestamp, treat as infinitely old
    }

    /// Format age for display
    pub fn format_age(&self) -> String {
        let secs = self.age_secs();
        if secs == u64::MAX {
            return "unknown".to_string();
        }
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

    /// Check if snapshot is stale (> 15 minutes by default)
    pub fn is_stale(&self) -> bool {
        self.age_secs() > STALE_THRESHOLD_SECS
    }

    /// Check if snapshot is recent (< 5 minutes)
    pub fn is_recent(&self) -> bool {
        self.age_secs() < 300
    }

    /// Update LLM bootstrap state from BootstrapState
    pub fn set_llm_state(&mut self, state: &crate::model_selection::BootstrapState) {
        self.llm_bootstrap_phase = Some(state.phase.to_string());
        self.llm_translator_model = state.translator.as_ref().map(|s| s.model.clone());
        self.llm_junior_model = state.junior.as_ref().map(|s| s.model.clone());
        self.llm_hardware_tier = state.hardware.as_ref().map(|h| h.tier.to_string());
        self.llm_error = state.error.clone();

        // Download progress
        if let Some(ref progress) = state.download_progress {
            self.llm_downloading_model = Some(progress.model.clone());
            self.llm_download_percent = Some(progress.percent());
            self.llm_download_speed = Some(progress.speed_bytes_per_sec);
            self.llm_download_eta_secs = progress.eta_seconds;
        } else {
            self.llm_downloading_model = None;
            self.llm_download_percent = None;
            self.llm_download_speed = None;
            self.llm_download_eta_secs = None;
        }
    }

    /// Check if LLM is ready
    pub fn is_llm_ready(&self) -> bool {
        self.llm_bootstrap_phase.as_deref() == Some("ready")
    }

    /// Format LLM status for display
    pub fn format_llm_status(&self) -> String {
        match self.llm_bootstrap_phase.as_deref() {
            Some("ready") => {
                let translator = self.llm_translator_model.as_deref().unwrap_or("none");
                let junior = self.llm_junior_model.as_deref().unwrap_or("none");
                format!("ready (translator: {}, junior: {})", translator, junior)
            }
            Some("pulling_models") => {
                let model = self.llm_downloading_model.as_deref().unwrap_or("unknown");
                let percent = self.llm_download_percent.unwrap_or(0.0);
                let eta = self.llm_download_eta_secs
                    .map(|s| format!(" ETA {}s", s))
                    .unwrap_or_default();
                format!("pulling {} ({:.1}%{})", model, percent, eta)
            }
            Some("benchmarking") => "benchmarking models".to_string(),
            Some("detecting_ollama") => "detecting ollama".to_string(),
            Some("installing_ollama") => "installing ollama".to_string(),
            Some("error") => {
                let err = self.llm_error.as_deref().unwrap_or("unknown error");
                format!("error: {}", err)
            }
            Some(phase) => phase.to_string(),
            None => "not initialized".to_string(),
        }
    }

    /// Update helper tracking state (v0.0.9)
    pub fn set_helpers_state(&mut self, summary: &crate::helpers::HelpersSummary) {
        self.helpers_total = summary.total;
        self.helpers_present = summary.present;
        self.helpers_missing = summary.missing;
        self.helpers_anna_installed = summary.installed_by_anna;
        self.helpers = summary.helpers.clone();
    }

    /// Format helpers status for display
    pub fn format_helpers_status(&self) -> String {
        format!(
            "{} present, {} missing ({} installed by Anna)",
            self.helpers_present,
            self.helpers_missing,
            self.helpers_anna_installed
        )
    }
}

/// Snapshot status for display
#[derive(Debug, Clone)]
pub enum SnapshotStatus {
    /// Snapshot available and recent
    Available { age_secs: u64, seq: u64 },
    /// Snapshot exists but is stale
    Stale { age_secs: u64, seq: u64 },
    /// No snapshot found
    Missing,
}

impl SnapshotStatus {
    pub fn from_snapshot(snapshot: &Option<StatusSnapshot>) -> Self {
        match snapshot {
            Some(s) => {
                let age = s.age_secs();
                if age > STALE_THRESHOLD_SECS {
                    SnapshotStatus::Stale { age_secs: age, seq: s.seq }
                } else {
                    SnapshotStatus::Available { age_secs: age, seq: s.seq }
                }
            }
            None => SnapshotStatus::Missing,
        }
    }

    pub fn format(&self) -> String {
        match self {
            SnapshotStatus::Available { age_secs, seq } => {
                if *age_secs < 60 {
                    format!("available ({}s ago, seq {})", age_secs, seq)
                } else {
                    format!("available ({}m ago, seq {})", age_secs / 60, seq)
                }
            }
            SnapshotStatus::Stale { age_secs, seq } => {
                if *age_secs < 3600 {
                    format!("stale ({}m old, seq {})", age_secs / 60, seq)
                } else {
                    format!("stale ({}h old, seq {})", age_secs / 3600, seq)
                }
            }
            SnapshotStatus::Missing => "missing".to_string(),
        }
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
            generated_at: Some(Utc::now()),
            ..Default::default()
        };
        assert!(snapshot.age_secs() < 2);
        assert!(!snapshot.is_stale());
    }
}
