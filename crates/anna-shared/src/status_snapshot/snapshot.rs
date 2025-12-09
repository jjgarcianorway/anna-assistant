//! StatusSnapshot struct and methods (v0.0.211).

use serde::{Deserialize, Serialize};

use super::config::ConfigInfo;
use super::daemon::DaemonInfo;
use super::helpers_info::HelpersInfo;
use super::models::ModelsInfo;
use super::permissions::PermissionsInfo;
use super::update::{UpdateInfo, UpdateResult};
use super::version::VersionInfo;

/// Complete status snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusSnapshot {
    /// Timestamp when snapshot was taken (epoch seconds)
    pub captured_at_ts: u64,
    /// Version information
    pub versions: VersionInfo,
    /// Daemon health
    pub daemon: DaemonInfo,
    /// Permissions and access
    pub perms: PermissionsInfo,
    /// Update subsystem
    pub update: UpdateInfo,
    /// Helpers tracking
    pub helpers: HelpersInfo,
    /// Models subsystem
    pub models: ModelsInfo,
    /// Configuration
    pub config: ConfigInfo,
}

impl StatusSnapshot {
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

    /// Check if daemon is healthy
    pub fn daemon_healthy(&self) -> bool {
        self.daemon.running && self.daemon.last_error.is_none()
    }

    /// Check if models are ready
    pub fn models_ready(&self) -> bool {
        self.models.is_ready()
    }

    /// Check if update is available
    pub fn update_available(&self) -> bool {
        matches!(
            self.update.last_result,
            UpdateResult::UpdateAvailable { .. }
        )
    }

    /// Get overall health status string
    pub fn health_status(&self) -> &'static str {
        if !self.daemon.running {
            return "DAEMON_DOWN";
        }
        if self.daemon.last_error.is_some() {
            return "DAEMON_ERROR";
        }
        if !self.models.ollama_present {
            return "OLLAMA_MISSING";
        }
        if !self.models.ollama_running {
            return "OLLAMA_DOWN";
        }
        if !self.models_ready() {
            return "MODELS_PENDING";
        }
        "OK"
    }
}
