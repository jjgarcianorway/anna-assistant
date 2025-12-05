//! StatusSnapshot - single authoritative system state snapshot.
//!
//! Provides comprehensive, deterministic system state for annactl status.
//! All fields are optional where discovery can fail - no fiction.
//!
//! v0.0.29: Initial implementation.

use crate::helpers::{HelpersRegistry, InstallSource};
use crate::specialists::SpecialistRole;
use crate::teams::Team;
use serde::{Deserialize, Serialize};

/// Version information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionInfo {
    /// annactl version
    pub annactl: String,
    /// annad version
    pub annad: String,
    /// anna-shared version
    pub anna_shared: String,
    /// Current git tag (local)
    pub git_tag_current: Option<String>,
    /// Remote git tag (latest from GitHub)
    pub git_tag_remote: Option<String>,
}

impl VersionInfo {
    pub fn new(version: &str) -> Self {
        Self {
            annactl: version.to_string(),
            annad: version.to_string(),
            anna_shared: version.to_string(),
            git_tag_current: Some(format!("v{}", version)),
            git_tag_remote: None,
        }
    }

    pub fn with_remote(mut self, remote: Option<String>) -> Self {
        self.git_tag_remote = remote;
        self
    }
}

/// Daemon health status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaemonInfo {
    /// Whether daemon is running
    pub running: bool,
    /// Process ID if running
    pub pid: Option<u32>,
    /// Uptime in seconds
    pub uptime_s: Option<u64>,
    /// Last error message if any
    pub last_error: Option<String>,
}

impl DaemonInfo {
    pub fn running(pid: u32, uptime_s: u64) -> Self {
        Self {
            running: true,
            pid: Some(pid),
            uptime_s: Some(uptime_s),
            last_error: None,
        }
    }

    pub fn not_running() -> Self {
        Self {
            running: false,
            pid: None,
            uptime_s: None,
            last_error: None,
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.last_error = Some(error.into());
        self
    }
}

/// Permission and access information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionsInfo {
    /// Current user
    pub user: String,
    /// User groups
    pub groups: Vec<String>,
    /// Can connect to daemon socket
    pub can_talk_to_daemon: bool,
    /// Data directory is accessible
    pub data_dir_ok: bool,
}

impl PermissionsInfo {
    pub fn current() -> Self {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        Self {
            user,
            groups: Vec::new(), // Will be populated by caller
            can_talk_to_daemon: false,
            data_dir_ok: false,
        }
    }

    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = groups;
        self
    }

    pub fn with_daemon_access(mut self, can_talk: bool) -> Self {
        self.can_talk_to_daemon = can_talk;
        self
    }

    pub fn with_data_dir_ok(mut self, ok: bool) -> Self {
        self.data_dir_ok = ok;
        self
    }
}

/// Update check result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateResult {
    /// Up to date
    UpToDate,
    /// Update available
    UpdateAvailable { version: String },
    /// Update downloaded
    Downloaded { version: String },
    /// Update installed
    Installed { version: String },
    /// Check failed
    Failed { reason: String },
    /// Not checked yet
    NotChecked,
}

impl Default for UpdateResult {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// Update subsystem information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Check interval in seconds
    pub interval_s: u64,
    /// Last check timestamp (epoch seconds)
    pub last_check_ts: Option<u64>,
    /// Next check timestamp (epoch seconds)
    pub next_check_ts: Option<u64>,
    /// Last check result
    pub last_result: UpdateResult,
}

impl UpdateInfo {
    pub fn new(interval_s: u64) -> Self {
        Self {
            interval_s,
            last_check_ts: None,
            next_check_ts: None,
            last_result: UpdateResult::NotChecked,
        }
    }
}

/// Helper package summary (lite version for snapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperPackageLite {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub source: InstallSource,
}

/// Helpers summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelpersInfo {
    /// Total helpers tracked
    pub total: usize,
    /// Count by source
    pub anna_installed: usize,
    pub user_installed: usize,
    pub bundled: usize,
    /// List of helpers
    pub list: Vec<HelperPackageLite>,
}

impl HelpersInfo {
    pub fn from_registry(registry: &HelpersRegistry) -> Self {
        let list: Vec<HelperPackageLite> = registry
            .packages
            .iter()
            .map(|p| HelperPackageLite {
                id: p.id.clone(),
                name: p.name.clone(),
                available: p.available,
                source: p.install_source,
            })
            .collect();

        Self {
            total: registry.len(),
            anna_installed: registry.anna_installed().len(),
            user_installed: registry.packages.iter().filter(|p| p.install_source == InstallSource::User).count(),
            bundled: registry.packages.iter().filter(|p| p.install_source == InstallSource::Bundled).count(),
            list,
        }
    }
}

/// Model download status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadStatus {
    pub name: String,
    pub downloading: bool,
    pub progress_pct: Option<f32>,
    pub error: Option<String>,
}

/// Role-model binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleModelBinding {
    pub team: Team,
    pub role: SpecialistRole,
    pub model_name: String,
    pub model_present: bool,
}

/// Models subsystem information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelsInfo {
    /// Ollama binary present
    pub ollama_present: bool,
    /// Ollama service running
    pub ollama_running: bool,
    /// Ollama version if available
    pub ollama_version: Option<String>,
    /// Role-model bindings
    pub roles: Vec<RoleModelBinding>,
    /// Active downloads
    pub downloads: Vec<ModelDownloadStatus>,
}

impl ModelsInfo {
    pub fn is_ready(&self) -> bool {
        self.ollama_present && self.ollama_running && self.roles.iter().all(|r| r.model_present)
    }

    pub fn missing_models(&self) -> Vec<&str> {
        self.roles
            .iter()
            .filter(|r| !r.model_present)
            .map(|r| r.model_name.as_str())
            .collect()
    }
}

/// Configuration summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Clean REPL mode (non-debug)
    pub repl_clean_mode: bool,
    /// Autonomy level (0-100)
    pub autonomy_level: u8,
}

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
        matches!(self.update.last_result, UpdateResult::UpdateAvailable { .. })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_new() {
        let v = VersionInfo::new("0.0.29");
        assert_eq!(v.annactl, "0.0.29");
        assert_eq!(v.git_tag_current, Some("v0.0.29".to_string()));
    }

    #[test]
    fn test_daemon_info_running() {
        let d = DaemonInfo::running(1234, 3600);
        assert!(d.running);
        assert_eq!(d.pid, Some(1234));
        assert_eq!(d.uptime_s, Some(3600));
    }

    #[test]
    fn test_daemon_info_not_running() {
        let d = DaemonInfo::not_running();
        assert!(!d.running);
        assert!(d.pid.is_none());
    }

    #[test]
    fn test_update_result_default() {
        let r = UpdateResult::default();
        assert_eq!(r, UpdateResult::NotChecked);
    }

    #[test]
    fn test_status_snapshot_health() {
        let mut snap = StatusSnapshot::new();

        // Initial state
        assert_eq!(snap.health_status(), "DAEMON_DOWN");

        // Daemon running
        snap.daemon = DaemonInfo::running(1234, 100);
        assert_eq!(snap.health_status(), "OLLAMA_MISSING");

        // Ollama present but not running
        snap.models.ollama_present = true;
        assert_eq!(snap.health_status(), "OLLAMA_DOWN");

        // Ollama running
        snap.models.ollama_running = true;
        assert_eq!(snap.health_status(), "OK");

        // With missing model
        snap.models.roles.push(RoleModelBinding {
            team: Team::General,
            role: SpecialistRole::Junior,
            model_name: "test".to_string(),
            model_present: false,
        });
        assert_eq!(snap.health_status(), "MODELS_PENDING");
    }

    #[test]
    fn test_models_info_missing() {
        let mut m = ModelsInfo::default();
        m.ollama_present = true;
        m.ollama_running = true;
        m.roles.push(RoleModelBinding {
            team: Team::Storage,
            role: SpecialistRole::Junior,
            model_name: "llama3.2".to_string(),
            model_present: false,
        });
        m.roles.push(RoleModelBinding {
            team: Team::Storage,
            role: SpecialistRole::Senior,
            model_name: "llama3.2".to_string(),
            model_present: true,
        });

        assert!(!m.is_ready());
        assert_eq!(m.missing_models(), vec!["llama3.2"]);
    }
}
