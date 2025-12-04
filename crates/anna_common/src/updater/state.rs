//! Update State Machine v0.0.73
//!
//! Single source of truth for update state with step-by-step persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

use super::UPDATE_STATE_FILE;

/// Update step in the state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStep {
    /// No update in progress
    Idle,
    /// Acquiring lock
    AcquireLock,
    /// Checking remote release
    CheckRemote,
    /// Comparing versions
    CompareVersions,
    /// Downloading assets
    DownloadAssets,
    /// Verifying assets (SHA256)
    VerifyAssets,
    /// Installing CLI binary
    InstallCli,
    /// Installing daemon binary
    InstallDaemon,
    /// Restarting daemon
    RestartDaemon,
    /// Running healthcheck
    Healthcheck,
    /// Releasing lock
    ReleaseLock,
    /// Rolling back due to failure
    Rollback,
}

impl Default for UpdateStep {
    fn default() -> Self {
        Self::Idle
    }
}

impl UpdateStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::AcquireLock => "acquire_lock",
            Self::CheckRemote => "check_remote",
            Self::CompareVersions => "compare_versions",
            Self::DownloadAssets => "download_assets",
            Self::VerifyAssets => "verify_assets",
            Self::InstallCli => "install_cli",
            Self::InstallDaemon => "install_daemon",
            Self::RestartDaemon => "restart_daemon",
            Self::Healthcheck => "healthcheck",
            Self::ReleaseLock => "release_lock",
            Self::Rollback => "rollback",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Idle | Self::ReleaseLock)
    }
}

/// Result of a step execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStepResult {
    /// Step completed successfully
    Success,
    /// No update available
    NoUpdate,
    /// Update completed
    Updated { version: String },
    /// Step failed
    Failed { reason: String },
    /// Rolled back
    RolledBack { reason: String },
}

impl Default for UpdateStepResult {
    fn default() -> Self {
        Self::Success
    }
}

/// Lock status for observability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockStatus {
    /// Whether lock is currently held
    pub owned: bool,
    /// PID of lock holder
    pub pid: Option<u32>,
    /// When lock was acquired
    pub acquired_at: Option<u64>,
}

/// Complete update state v0.0.73 - single source of truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStateV73 {
    // === Core version info ===
    /// Current CLI version (annactl)
    pub current_version: String,
    /// Current daemon version (annad) - may differ during update
    pub current_daemon_version: String,
    /// Target version (if update available/in progress)
    pub target_version: Option<String>,

    // === Update channel ===
    /// Channel: "stable" only for now
    pub channel: String,

    // === Timing ===
    /// Last check timestamp (RFC3339)
    #[serde(default)]
    pub last_check_time: Option<DateTime<Utc>>,
    /// Next check timestamp (RFC3339)
    #[serde(default)]
    pub next_check_time: Option<DateTime<Utc>>,
    /// Last successful update timestamp
    #[serde(default)]
    pub last_successful_update_time: Option<DateTime<Utc>>,

    // === Results ===
    /// Last result: no_update | update_available | updated | failed
    pub last_result: String,
    /// Last error message (if failed)
    #[serde(default)]
    pub last_error: Option<String>,

    // === State machine ===
    /// Current/last step
    pub last_step: UpdateStep,
    /// Step result
    #[serde(default)]
    pub step_result: UpdateStepResult,

    // === Lock status ===
    #[serde(default)]
    pub lock_status: LockStatus,

    // === Backup info for rollback ===
    #[serde(default)]
    pub backup_cli_path: Option<String>,
    #[serde(default)]
    pub backup_daemon_path: Option<String>,
    #[serde(default)]
    pub backup_cli_checksum: Option<String>,
    #[serde(default)]
    pub backup_daemon_checksum: Option<String>,

    // === Asset verification ===
    #[serde(default)]
    pub downloaded_cli_checksum: Option<String>,
    #[serde(default)]
    pub downloaded_daemon_checksum: Option<String>,
}

impl Default for UpdateStateV73 {
    fn default() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            current_daemon_version: env!("CARGO_PKG_VERSION").to_string(),
            target_version: None,
            channel: "stable".to_string(),
            last_check_time: None,
            next_check_time: None,
            last_successful_update_time: None,
            last_result: "pending".to_string(),
            last_error: None,
            last_step: UpdateStep::Idle,
            step_result: UpdateStepResult::Success,
            lock_status: LockStatus::default(),
            backup_cli_path: None,
            backup_daemon_path: None,
            backup_cli_checksum: None,
            backup_daemon_checksum: None,
            downloaded_cli_checksum: None,
            downloaded_daemon_checksum: None,
        }
    }
}

impl UpdateStateV73 {
    /// Load state from disk or create default
    pub fn load() -> Self {
        let path = Path::new(UPDATE_STATE_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    /// Save state to disk atomically
    pub fn save(&self) -> io::Result<()> {
        // Ensure directory exists
        if let Some(parent) = Path::new(UPDATE_STATE_FILE).parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        crate::atomic_write(UPDATE_STATE_FILE, &content)
    }

    /// Record step start
    pub fn start_step(&mut self, step: UpdateStep) -> io::Result<()> {
        self.last_step = step;
        self.step_result = UpdateStepResult::Success;
        self.save()
    }

    /// Record step success
    pub fn complete_step(&mut self) -> io::Result<()> {
        self.step_result = UpdateStepResult::Success;
        self.save()
    }

    /// Record step failure
    pub fn fail_step(&mut self, reason: &str) -> io::Result<()> {
        self.step_result = UpdateStepResult::Failed {
            reason: reason.to_string(),
        };
        self.last_error = Some(reason.to_string());
        self.last_result = "failed".to_string();
        self.save()
    }

    /// Record no update available
    pub fn record_no_update(&mut self) -> io::Result<()> {
        self.last_check_time = Some(Utc::now());
        self.last_result = "no_update".to_string();
        self.last_error = None;
        self.target_version = None;
        self.step_result = UpdateStepResult::NoUpdate;
        self.schedule_next_check(600); // 10 minutes
        self.save()
    }

    /// Record update available
    pub fn record_update_available(&mut self, version: &str) -> io::Result<()> {
        self.last_check_time = Some(Utc::now());
        self.last_result = "update_available".to_string();
        self.last_error = None;
        self.target_version = Some(version.to_string());
        self.save()
    }

    /// Record successful update
    pub fn record_updated(&mut self, version: &str) -> io::Result<()> {
        self.current_version = version.to_string();
        self.current_daemon_version = version.to_string();
        self.last_successful_update_time = Some(Utc::now());
        self.last_result = "updated".to_string();
        self.last_error = None;
        self.target_version = None;
        self.last_step = UpdateStep::Idle;
        self.step_result = UpdateStepResult::Updated {
            version: version.to_string(),
        };
        self.schedule_next_check(600);
        self.save()
    }

    /// Record rollback
    pub fn record_rollback(&mut self, reason: &str) -> io::Result<()> {
        self.last_result = "failed".to_string();
        self.last_error = Some(format!("Rolled back: {}", reason));
        self.target_version = None;
        self.last_step = UpdateStep::Idle;
        self.step_result = UpdateStepResult::RolledBack {
            reason: reason.to_string(),
        };
        self.schedule_next_check(600);
        self.save()
    }

    /// Schedule next check
    fn schedule_next_check(&mut self, seconds: i64) {
        self.next_check_time = Some(Utc::now() + chrono::Duration::seconds(seconds));
    }

    /// Check if update check is due
    pub fn is_check_due(&self) -> bool {
        match self.next_check_time {
            Some(next) => Utc::now() >= next,
            None => true, // Never checked
        }
    }

    /// Check for version mismatch between CLI and daemon
    pub fn has_version_mismatch(&self) -> bool {
        self.current_version != self.current_daemon_version
    }

    /// Format for human-readable status
    pub fn format_status(&self) -> String {
        let check_str = self
            .last_check_time
            .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "never".to_string());

        let next_str = self
            .next_check_time
            .map(|t| {
                let now = Utc::now();
                if t > now {
                    let mins = (t - now).num_minutes();
                    format!("in {} min", mins)
                } else {
                    "now".to_string()
                }
            })
            .unwrap_or_else(|| "not scheduled".to_string());

        format!(
            "Last: {} ({}) | Next: {}",
            check_str, self.last_result, next_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = UpdateStateV73::default();
        assert_eq!(state.channel, "stable");
        assert_eq!(state.last_step, UpdateStep::Idle);
        assert!(!state.has_version_mismatch());
    }

    #[test]
    fn test_step_as_str() {
        assert_eq!(UpdateStep::AcquireLock.as_str(), "acquire_lock");
        assert_eq!(UpdateStep::DownloadAssets.as_str(), "download_assets");
        assert_eq!(UpdateStep::Rollback.as_str(), "rollback");
    }

    #[test]
    fn test_terminal_steps() {
        assert!(UpdateStep::Idle.is_terminal());
        assert!(UpdateStep::ReleaseLock.is_terminal());
        assert!(!UpdateStep::DownloadAssets.is_terminal());
    }

    #[test]
    fn test_version_mismatch_detection() {
        let mut state = UpdateStateV73::default();
        assert!(!state.has_version_mismatch());

        state.current_daemon_version = "0.0.99".to_string();
        assert!(state.has_version_mismatch());
    }

    #[test]
    fn test_check_due() {
        let mut state = UpdateStateV73::default();
        // No next check scheduled = check is due
        assert!(state.is_check_due());

        // Schedule in the future
        state.next_check_time = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(!state.is_check_due());

        // Schedule in the past
        state.next_check_time = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(state.is_check_due());
    }
}
