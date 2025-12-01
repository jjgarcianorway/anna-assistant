//! Anna Configuration v6.0.2 - Auto-Update Visibility
//!
//! Simplified system configuration for the telemetry daemon.
//! No LLM config - pure system monitoring.
//!
//! Configuration lives in /etc/anna/config.toml
//!
//! v6.0.2: Added auto-update configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// System configuration directory
pub const SYSTEM_CONFIG_DIR: &str = "/etc/anna";
const CONFIG_FILE: &str = "config.toml";

/// Anna data directory (for knowledge, telemetry, logs)
pub const DATA_DIR: &str = "/var/lib/anna";

/// Core mode for Anna operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CoreMode {
    #[default]
    Normal,
    Dev,
}

impl CoreMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoreMode::Normal => "normal",
            CoreMode::Dev => "dev",
        }
    }
}

/// Telemetry settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    /// How often to sample processes (seconds)
    #[serde(default = "default_sample_interval")]
    pub sample_interval_secs: u64,

    /// How often to scan logs (seconds)
    #[serde(default = "default_log_scan_interval")]
    pub log_scan_interval_secs: u64,

    /// Maximum events to keep per log file
    #[serde(default = "default_max_events")]
    pub max_events_per_log: usize,

    /// Days to retain event logs
    #[serde(default = "default_retention_days")]
    pub retention_days: u64,
}

fn default_sample_interval() -> u64 {
    15 // 15 seconds
}

fn default_log_scan_interval() -> u64 {
    60 // 1 minute
}

fn default_max_events() -> usize {
    100_000 // 100k events per log
}

fn default_retention_days() -> u64 {
    30 // 30 days
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            sample_interval_secs: default_sample_interval(),
            log_scan_interval_secs: default_log_scan_interval(),
            max_events_per_log: default_max_events(),
            retention_days: default_retention_days(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

/// Core configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(default)]
    pub mode: CoreMode,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            mode: CoreMode::Normal,
        }
    }
}

/// Auto-update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Whether auto-update is enabled
    #[serde(default = "default_update_enabled")]
    pub enabled: bool,

    /// Check interval in minutes
    #[serde(default = "default_update_interval")]
    pub interval_minutes: u64,
}

fn default_update_enabled() -> bool {
    true
}

fn default_update_interval() -> u64 {
    10 // 10 minutes
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enabled: default_update_enabled(),
            interval_minutes: default_update_interval(),
        }
    }
}

/// Auto-update state (runtime, stored in data dir)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateState {
    /// Last check timestamp (unix seconds)
    pub last_check_at: u64,
    /// Last check result
    pub last_result: String,
    /// Current version
    pub current_version: String,
    /// Latest available version (if known)
    pub latest_version: Option<String>,
    /// Next scheduled check (unix seconds)
    pub next_check_at: u64,
}

impl UpdateState {
    const STATE_FILE: &'static str = "update_state.json";

    pub fn load() -> Self {
        let path = PathBuf::from(DATA_DIR).join(Self::STATE_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(DATA_DIR).join(Self::STATE_FILE);
        let path_str = path.to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(path_str, &content)
    }

    /// Format last check time for display
    pub fn format_last_check(&self) -> String {
        if self.last_check_at == 0 {
            return "never".to_string();
        }
        crate::format_time_ago(self.last_check_at)
    }

    /// Format next check time for display
    pub fn format_next_check(&self) -> String {
        if self.next_check_at == 0 {
            return "n/a".to_string();
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if self.next_check_at <= now {
            return "now".to_string();
        }
        let secs = self.next_check_at - now;
        if secs < 60 {
            format!("in {}s", secs)
        } else {
            format!("in {}m", secs / 60)
        }
    }
}

/// Complete Anna configuration v6.0.2
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnaConfig {
    #[serde(default)]
    pub core: CoreConfig,

    #[serde(default)]
    pub telemetry: TelemetrySettings,

    #[serde(default)]
    pub log: LogConfig,

    #[serde(default)]
    pub update: UpdateConfig,
}

impl AnnaConfig {
    /// Load config from system config directory (/etc/anna/config.toml)
    pub fn load() -> Self {
        let system_path = config_path();
        if system_path.exists() {
            if let Ok(content) = fs::read_to_string(&system_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Save config to system config file
    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }
}

/// Get the config file path
pub fn config_path() -> PathBuf {
    PathBuf::from(SYSTEM_CONFIG_DIR).join(CONFIG_FILE)
}

/// Get the config directory
pub fn config_dir() -> PathBuf {
    PathBuf::from(SYSTEM_CONFIG_DIR)
}

/// Get the data directory
pub fn data_dir() -> PathBuf {
    PathBuf::from(DATA_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnnaConfig::default();
        assert_eq!(config.core.mode, CoreMode::Normal);
        assert_eq!(config.telemetry.sample_interval_secs, 15);
        assert_eq!(config.telemetry.log_scan_interval_secs, 60);
    }

    #[test]
    fn test_toml_serialization() {
        let config = AnnaConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[core]"));
        assert!(toml_str.contains("[telemetry]"));
    }
}
