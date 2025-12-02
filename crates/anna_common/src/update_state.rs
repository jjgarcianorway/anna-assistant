//! Update State v7.31.0 - Truthful Auto-Update Status
//!
//! Stores and tracks update scheduler state for transparent reporting.
//! State file: /var/lib/anna/internal/update_state.json

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Path to update state file
pub const UPDATE_STATE_PATH: &str = "/var/lib/anna/internal/update_state.json";

/// Update check modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateMode {
    /// Auto-check enabled
    Auto,
    /// Manual checks only
    Manual,
    /// Updates disabled
    Disabled,
}

impl Default for UpdateMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Result of last update check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateResult {
    /// Check succeeded, no updates
    NoUpdates,
    /// Check succeeded, updates available
    UpdatesAvailable { count: u32 },
    /// Check failed
    Failed { error: String },
    /// Never checked
    Pending,
}

impl Default for UpdateResult {
    fn default() -> Self {
        Self::Pending
    }
}

/// What is being auto-checked
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateTarget {
    /// Anna releases from GitHub
    AnnaRelease,
    /// Pacman packages
    PacmanPackages,
    /// Both
    Both,
}

impl Default for UpdateTarget {
    fn default() -> Self {
        Self::AnnaRelease
    }
}

/// Update scheduler state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    /// What is being checked
    pub target: UpdateTarget,
    /// Update mode
    pub mode: UpdateMode,
    /// Check interval in seconds
    pub interval_secs: u64,
    /// Last check timestamp (epoch)
    pub last_check_epoch: Option<u64>,
    /// Last check result
    pub last_result: UpdateResult,
    /// Next scheduled check (epoch)
    pub next_check_epoch: Option<u64>,
    /// State file version
    pub version: u32,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            target: UpdateTarget::default(),
            mode: UpdateMode::default(),
            interval_secs: 600,  // 10 minutes
            last_check_epoch: None,
            last_result: UpdateResult::default(),
            next_check_epoch: None,
            version: 1,
        }
    }
}

impl UpdateState {
    /// Load state from disk
    pub fn load() -> Self {
        let path = Path::new(UPDATE_STATE_PATH);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save state to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Path::new(UPDATE_STATE_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Record a check result
    pub fn record_check(&mut self, result: UpdateResult) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.last_check_epoch = Some(now);
        self.last_result = result;
        self.next_check_epoch = Some(now + self.interval_secs);
    }

    /// Format last check time for display
    pub fn format_last_check(&self) -> String {
        match self.last_check_epoch {
            Some(epoch) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let age = now.saturating_sub(epoch);
                format_age(age)
            }
            None => "never".to_string(),
        }
    }

    /// Format next check time for display
    pub fn format_next_check(&self) -> String {
        match self.next_check_epoch {
            Some(epoch) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if epoch > now {
                    let until = epoch - now;
                    format!("in {}", format_duration(until))
                } else {
                    "pending".to_string()
                }
            }
            None => "not scheduled".to_string(),
        }
    }

    /// Format result for display
    pub fn format_result(&self) -> String {
        match &self.last_result {
            UpdateResult::NoUpdates => "no updates".to_string(),
            UpdateResult::UpdatesAvailable { count } => format!("{} update(s) available", count),
            UpdateResult::Failed { error } => format!("failed: {}", error),
            UpdateResult::Pending => "pending".to_string(),
        }
    }

    /// Format mode for display
    pub fn format_mode(&self) -> String {
        match self.mode {
            UpdateMode::Auto => "auto".to_string(),
            UpdateMode::Manual => "manual".to_string(),
            UpdateMode::Disabled => "disabled".to_string(),
        }
    }

    /// Format interval for display
    pub fn format_interval(&self) -> String {
        format_duration(self.interval_secs)
    }
}

/// Format age in human-readable form
fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Format duration in human-readable form
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

/// Check if daemon is running
pub fn is_daemon_running() -> bool {
    // Check via systemctl
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "annad"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = UpdateState::default();
        assert_eq!(state.mode, UpdateMode::Auto);
        assert_eq!(state.interval_secs, 600);
        assert!(state.last_check_epoch.is_none());
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age(30), "30s ago");
        assert_eq!(format_age(120), "2m ago");
        assert_eq!(format_age(7200), "2h ago");
    }
}
