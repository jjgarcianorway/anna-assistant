//! Anna Configuration v5.3.0 - Telemetry Core
//!
//! Simplified system configuration for the telemetry daemon.
//! No LLM config - pure system monitoring.
//!
//! Configuration lives in /etc/anna/config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// Complete Anna configuration v5.3.0
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnaConfig {
    #[serde(default)]
    pub core: CoreConfig,

    #[serde(default)]
    pub telemetry: TelemetrySettings,

    #[serde(default)]
    pub log: LogConfig,
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
