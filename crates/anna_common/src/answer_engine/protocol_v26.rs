//! Protocol v0.26.0 - Auto-update Reliability, Self-Healing, Logging
//!
//! Core types for reliable auto-updates and daemon self-healing.

use serde::{Deserialize, Serialize};

/// Protocol version
pub const PROTOCOL_VERSION_V26: &str = "0.26.0";

// ============================================================================
// AUTO-UPDATE TYPES
// ============================================================================

/// Update download state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    /// Not downloading
    Idle,
    /// Checking for updates
    Checking,
    /// Downloading in background
    Downloading { progress_percent: u8 },
    /// Download complete, ready to install
    Ready,
    /// Installing update
    Installing,
    /// Update complete, restart needed
    PendingRestart,
    /// Download failed
    Failed { reason: String },
}

impl Default for DownloadState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Update installation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InstallStrategy {
    /// Zero-downtime: prepare new binary, atomic swap
    #[default]
    ZeroDowntime,
    /// Stop, replace, restart (brief downtime)
    StopAndReplace,
    /// Manual: download only, user installs
    Manual,
}

/// Update progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    /// Current state
    pub state: DownloadState,
    /// Target version
    pub target_version: Option<String>,
    /// Download progress (bytes)
    pub bytes_downloaded: u64,
    /// Total size (bytes)
    pub total_bytes: Option<u64>,
    /// Download started timestamp
    pub started_at: Option<i64>,
    /// Estimated completion timestamp
    pub eta_seconds: Option<u64>,
    /// Checksum verification status
    pub checksum_verified: Option<bool>,
}

impl Default for UpdateProgress {
    fn default() -> Self {
        Self {
            state: DownloadState::Idle,
            target_version: None,
            bytes_downloaded: 0,
            total_bytes: None,
            started_at: None,
            eta_seconds: None,
            checksum_verified: None,
        }
    }
}

impl UpdateProgress {
    /// Create new progress for a version
    pub fn for_version(version: &str) -> Self {
        Self {
            target_version: Some(version.to_string()),
            started_at: Some(chrono::Utc::now().timestamp()),
            ..Default::default()
        }
    }

    /// Progress percentage (0-100)
    pub fn percentage(&self) -> u8 {
        if let Some(total) = self.total_bytes {
            if total > 0 {
                return ((self.bytes_downloaded as f64 / total as f64) * 100.0) as u8;
            }
        }
        0
    }

    /// Check if download is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Ready | DownloadState::PendingRestart
        )
    }

    /// Check if failed
    pub fn is_failed(&self) -> bool {
        matches!(self.state, DownloadState::Failed { .. })
    }
}

/// Binary checksum information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryChecksum {
    /// Binary name (annad, annactl)
    pub binary: String,
    /// SHA256 checksum
    pub sha256: String,
    /// File size
    pub size: u64,
    /// Verified flag
    pub verified: bool,
}

/// Update event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvent {
    /// Event timestamp
    pub timestamp: i64,
    /// Event type
    pub event_type: UpdateEventType,
    /// Related version
    pub version: Option<String>,
    /// Details
    pub details: Option<String>,
}

/// Update event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateEventType {
    /// Started checking for updates
    CheckStarted,
    /// Found new version
    NewVersionFound,
    /// No update available
    NoUpdateAvailable,
    /// Download started
    DownloadStarted,
    /// Download progress
    DownloadProgress,
    /// Download completed
    DownloadCompleted,
    /// Checksum verification passed
    ChecksumVerified,
    /// Checksum verification failed
    ChecksumFailed,
    /// Installation started
    InstallStarted,
    /// Installation completed
    InstallCompleted,
    /// Installation failed
    InstallFailed,
    /// Daemon restart requested
    RestartRequested,
    /// Update check failed
    CheckFailed,
    /// Download failed
    DownloadFailed,
}

// ============================================================================
// SELF-HEALING TYPES
// ============================================================================

/// Daemon watchdog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    /// Enable watchdog
    pub enabled: bool,
    /// Health check interval (seconds)
    pub check_interval_secs: u64,
    /// Max consecutive failures before restart
    pub max_failures: u32,
    /// Restart cooldown (seconds)
    pub restart_cooldown_secs: u64,
    /// Max restarts per hour
    pub max_restarts_per_hour: u32,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 30,
            max_failures: 3,
            restart_cooldown_secs: 60,
            max_restarts_per_hour: 5,
        }
    }
}

/// Watchdog state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchdogState {
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Last health check timestamp
    pub last_check: Option<i64>,
    /// Last successful check timestamp
    pub last_success: Option<i64>,
    /// Restarts in current hour
    pub restarts_this_hour: u32,
    /// Hour counter reset timestamp
    pub hour_started: Option<i64>,
    /// Last restart timestamp
    pub last_restart: Option<i64>,
}

impl WatchdogState {
    /// Record a successful health check
    pub fn record_success(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.consecutive_failures = 0;
        self.last_check = Some(now);
        self.last_success = Some(now);
    }

    /// Record a failed health check
    pub fn record_failure(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.consecutive_failures += 1;
        self.last_check = Some(now);
    }

    /// Record a restart
    pub fn record_restart(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.last_restart = Some(now);
        self.consecutive_failures = 0;

        // Reset hour counter if needed
        let hour_ago = now - 3600;
        if self.hour_started.map(|h| h < hour_ago).unwrap_or(true) {
            self.hour_started = Some(now);
            self.restarts_this_hour = 0;
        }
        self.restarts_this_hour += 1;
    }

    /// Check if restart is allowed
    pub fn can_restart(&self, config: &WatchdogConfig) -> bool {
        let now = chrono::Utc::now().timestamp();

        // Check cooldown
        if let Some(last) = self.last_restart {
            if now - last < config.restart_cooldown_secs as i64 {
                return false;
            }
        }

        // Check hourly limit
        if self.restarts_this_hour >= config.max_restarts_per_hour {
            // Reset if hour passed
            if let Some(hour_start) = self.hour_started {
                if now - hour_start < 3600 {
                    return false;
                }
            }
        }

        true
    }

    /// Check if restart is needed
    pub fn needs_restart(&self, config: &WatchdogConfig) -> bool {
        self.consecutive_failures >= config.max_failures && self.can_restart(config)
    }
}

/// Self-healing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingEvent {
    /// Event timestamp
    pub timestamp: i64,
    /// Event type
    pub event_type: HealingEventType,
    /// Component affected
    pub component: String,
    /// Action taken
    pub action: Option<String>,
    /// Success flag
    pub success: bool,
    /// Details
    pub details: Option<String>,
}

/// Self-healing event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealingEventType {
    /// Health check performed
    HealthCheck,
    /// Component degraded
    ComponentDegraded,
    /// Component failed
    ComponentFailed,
    /// Component recovered
    ComponentRecovered,
    /// Restart initiated
    RestartInitiated,
    /// Restart completed
    RestartCompleted,
    /// Restart failed
    RestartFailed,
    /// Repair attempted
    RepairAttempted,
    /// Repair succeeded
    RepairSucceeded,
    /// Repair failed
    RepairFailed,
    /// Rate limit hit
    RateLimitHit,
}

// ============================================================================
// LOGGING TYPES
// ============================================================================

/// Extended log component for v0.26.0
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogComponentV26 {
    /// Core daemon operations
    Daemon,
    /// Update system
    Update,
    /// Self-healing / watchdog
    Watchdog,
    /// Health checks
    Health,
    /// Repair actions
    Repair,
    /// LLM operations
    Llm,
    /// Probe execution
    Probe,
    /// IPC communication
    Ipc,
    /// Request handling
    Request,
    /// Configuration
    Config,
}

/// Structured trace for update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTrace {
    /// Trace ID
    pub trace_id: String,
    /// Start timestamp
    pub started_at: i64,
    /// End timestamp
    pub ended_at: Option<i64>,
    /// Current/final state
    pub state: DownloadState,
    /// Version being updated from
    pub from_version: String,
    /// Version being updated to
    pub to_version: String,
    /// Events in this trace
    pub events: Vec<UpdateEvent>,
    /// Final result
    pub result: Option<UpdateResultV26>,
}

/// Update result summary (v26 version to avoid conflict with types.rs)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateResultV26 {
    /// Update successful
    Success,
    /// Update failed with reason
    Failed { reason: String },
    /// Update cancelled
    Cancelled,
    /// Update skipped (already up to date)
    Skipped,
}

/// Structured trace for self-healing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingTrace {
    /// Trace ID
    pub trace_id: String,
    /// Trigger timestamp
    pub triggered_at: i64,
    /// Completion timestamp
    pub completed_at: Option<i64>,
    /// Component being healed
    pub component: String,
    /// Events in this trace
    pub events: Vec<HealingEvent>,
    /// Final success
    pub success: bool,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_state_default() {
        let state = DownloadState::default();
        assert!(matches!(state, DownloadState::Idle));
    }

    #[test]
    fn test_update_progress_percentage() {
        let mut progress = UpdateProgress::default();
        progress.total_bytes = Some(1000);
        progress.bytes_downloaded = 500;
        assert_eq!(progress.percentage(), 50);
    }

    #[test]
    fn test_update_progress_complete() {
        let mut progress = UpdateProgress::default();
        progress.state = DownloadState::Ready;
        assert!(progress.is_complete());

        progress.state = DownloadState::Downloading { progress_percent: 50 };
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_watchdog_config_default() {
        let config = WatchdogConfig::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval_secs, 30);
        assert_eq!(config.max_failures, 3);
    }

    #[test]
    fn test_watchdog_state_record_success() {
        let mut state = WatchdogState::default();
        state.consecutive_failures = 5;
        state.record_success();
        assert_eq!(state.consecutive_failures, 0);
        assert!(state.last_success.is_some());
    }

    #[test]
    fn test_watchdog_state_record_failure() {
        let mut state = WatchdogState::default();
        state.record_failure();
        state.record_failure();
        assert_eq!(state.consecutive_failures, 2);
    }

    #[test]
    fn test_watchdog_needs_restart() {
        let config = WatchdogConfig {
            max_failures: 3,
            restart_cooldown_secs: 0,
            ..Default::default()
        };
        let mut state = WatchdogState::default();

        // Not enough failures
        state.consecutive_failures = 2;
        assert!(!state.needs_restart(&config));

        // Enough failures
        state.consecutive_failures = 3;
        assert!(state.needs_restart(&config));
    }

    #[test]
    fn test_watchdog_restart_cooldown() {
        let config = WatchdogConfig {
            max_failures: 1,
            restart_cooldown_secs: 3600,
            ..Default::default()
        };
        let mut state = WatchdogState::default();
        state.consecutive_failures = 5;
        state.last_restart = Some(chrono::Utc::now().timestamp());

        // In cooldown - can't restart
        assert!(!state.can_restart(&config));
    }

    #[test]
    fn test_update_event_types() {
        let event = UpdateEvent {
            timestamp: chrono::Utc::now().timestamp(),
            event_type: UpdateEventType::NewVersionFound,
            version: Some("0.26.0".to_string()),
            details: None,
        };
        assert_eq!(event.event_type, UpdateEventType::NewVersionFound);
    }

    #[test]
    fn test_healing_event_types() {
        let event = HealingEvent {
            timestamp: chrono::Utc::now().timestamp(),
            event_type: HealingEventType::RestartCompleted,
            component: "daemon".to_string(),
            action: Some("systemctl restart annad".to_string()),
            success: true,
            details: None,
        };
        assert!(event.success);
    }

    #[test]
    fn test_install_strategy_default() {
        let strategy = InstallStrategy::default();
        assert!(matches!(strategy, InstallStrategy::ZeroDowntime));
    }
}
