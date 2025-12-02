//! Anna Configuration v7.29.0 - Auto-Update Scheduler & KDB Cache
//!
//! Simplified system configuration for the telemetry daemon.
//! No LLM config - pure system monitoring.
//!
//! Configuration lives in /etc/anna/config.toml
//!
//! v6.0.2: Added auto-update configuration
//! v7.6.0: Added telemetry enable/disable, max_keys limit
//! v7.26.0: Added instrumentation settings (AUR gate, auto-install)
//! v7.29.0: Enhanced update scheduler with mode/interval, KDB cache TTLs

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

/// Telemetry settings (v7.6.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    /// Whether telemetry collection is enabled
    #[serde(default = "default_telemetry_enabled")]
    pub enabled: bool,

    /// How often to sample processes (seconds, valid: 5-300)
    #[serde(default = "default_sample_interval")]
    pub sample_interval_secs: u64,

    /// How often to scan logs (seconds)
    #[serde(default = "default_log_scan_interval")]
    pub log_scan_interval_secs: u64,

    /// Maximum events to keep per log file
    #[serde(default = "default_max_events")]
    pub max_events_per_log: usize,

    /// Days to retain telemetry data (valid: 1-365)
    #[serde(default = "default_retention_days")]
    pub retention_days: u64,

    /// Maximum distinct keys to track (valid: 100-50000)
    #[serde(default = "default_max_keys")]
    pub max_keys: usize,
}

fn default_telemetry_enabled() -> bool {
    true
}

fn default_sample_interval() -> u64 {
    30 // 30 seconds (v7.6.0: changed from 15)
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

fn default_max_keys() -> usize {
    5000 // 5000 distinct keys
}

impl TelemetrySettings {
    /// Validate and clamp sample_interval_secs to valid range (5-300)
    pub fn effective_sample_interval(&self) -> u64 {
        self.sample_interval_secs.clamp(5, 300)
    }

    /// Validate and clamp retention_days to valid range (1-365)
    pub fn effective_retention_days(&self) -> u64 {
        self.retention_days.clamp(1, 365)
    }

    /// Validate and clamp max_keys to valid range (100-50000)
    pub fn effective_max_keys(&self) -> usize {
        self.max_keys.clamp(100, 50000)
    }

    /// Check if sample_interval was clamped
    pub fn sample_interval_was_clamped(&self) -> bool {
        self.sample_interval_secs != self.effective_sample_interval()
    }

    /// Check if retention_days was clamped
    pub fn retention_days_was_clamped(&self) -> bool {
        self.retention_days != self.effective_retention_days()
    }

    /// Check if max_keys was clamped
    pub fn max_keys_was_clamped(&self) -> bool {
        self.max_keys != self.effective_max_keys()
    }
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            enabled: default_telemetry_enabled(),
            sample_interval_secs: default_sample_interval(),
            log_scan_interval_secs: default_log_scan_interval(),
            max_events_per_log: default_max_events(),
            retention_days: default_retention_days(),
            max_keys: default_max_keys(),
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

/// Auto-update mode (v7.29.0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateMode {
    #[default]
    Auto,
    Manual,
}

impl UpdateMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateMode::Auto => "auto",
            UpdateMode::Manual => "manual",
        }
    }
}

/// Auto-update configuration (v7.29.0 enhanced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Update mode: auto or manual (v7.29.0)
    #[serde(default)]
    pub mode: UpdateMode,

    /// Check interval in seconds (default 600 = 10 minutes)
    #[serde(default = "default_update_interval_seconds")]
    pub interval_seconds: u64,

    /// Maximum backoff on failure (6 hours)
    #[serde(default = "default_max_backoff_seconds")]
    pub max_backoff_seconds: u64,
}

fn default_update_interval_seconds() -> u64 {
    600 // 10 minutes
}

fn default_max_backoff_seconds() -> u64 {
    21600 // 6 hours
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            mode: UpdateMode::Auto,
            interval_seconds: default_update_interval_seconds(),
            max_backoff_seconds: default_max_backoff_seconds(),
        }
    }
}

/// KDB cache TTL settings (v7.29.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Software/hardware inventory TTL (seconds, default 600 = 10 minutes)
    #[serde(default = "default_inventory_ttl")]
    pub inventory_ttl_seconds: u64,

    /// Telemetry rollup TTL (seconds, default 60)
    #[serde(default = "default_telemetry_ttl")]
    pub telemetry_ttl_seconds: u64,
}

fn default_inventory_ttl() -> u64 {
    600 // 10 minutes
}

fn default_telemetry_ttl() -> u64 {
    60 // 1 minute
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            inventory_ttl_seconds: default_inventory_ttl(),
            telemetry_ttl_seconds: default_telemetry_ttl(),
        }
    }
}

/// Instrumentation settings (v7.26.0)
/// Controls auto-install behavior for system probes/tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationSettings {
    /// Whether auto-install is enabled (default: true)
    #[serde(default = "default_auto_install_enabled")]
    pub auto_install_enabled: bool,

    /// Allow AUR packages (default: false - requires explicit enable)
    #[serde(default)]
    pub allow_aur: bool,

    /// Rate limit: max installs per 24 hours (default: 1)
    #[serde(default = "default_max_installs_per_day")]
    pub max_installs_per_day: u32,
}

fn default_auto_install_enabled() -> bool {
    true
}

fn default_max_installs_per_day() -> u32 {
    1
}

impl Default for InstrumentationSettings {
    fn default() -> Self {
        Self {
            auto_install_enabled: default_auto_install_enabled(),
            allow_aur: false,
            max_installs_per_day: default_max_installs_per_day(),
        }
    }
}

/// Update check result (v7.29.0)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdateResult {
    #[default]
    Pending,
    Ok,
    UpdateAvailable,
    Failed,
}

impl UpdateResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateResult::Pending => "pending",
            UpdateResult::Ok => "ok",
            UpdateResult::UpdateAvailable => "update available",
            UpdateResult::Failed => "failed",
        }
    }
}

/// Auto-update state (v7.29.0 enhanced, stored in /var/lib/anna/internal/state.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateState {
    /// Last check timestamp (unix epoch seconds)
    pub last_check_epoch: u64,
    /// Next scheduled check (unix epoch seconds)
    pub next_check_epoch: u64,
    /// Last check result
    #[serde(default)]
    pub last_result: UpdateResult,
    /// Last failure reason (if failed)
    pub last_failure_reason: Option<String>,
    /// Current consecutive failure count (for backoff)
    #[serde(default)]
    pub consecutive_failures: u32,
    /// Current version
    pub current_version: String,
    /// Latest available version (if known)
    pub latest_version: Option<String>,
}

impl UpdateState {
    /// State file path (v7.29.0: moved to internal/)
    pub fn state_path() -> PathBuf {
        PathBuf::from(DATA_DIR).join("internal/state.json")
    }

    pub fn load() -> Self {
        let path = Self::state_path();
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
        let path = Self::state_path();
        // Ensure internal directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let path_str = path.to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(path_str, &content)
    }

    /// Get current time as epoch seconds
    pub fn now_epoch() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Record a successful check
    pub fn record_success(&mut self, latest_version: Option<String>) {
        self.last_check_epoch = Self::now_epoch();
        self.consecutive_failures = 0;
        self.last_failure_reason = None;
        if latest_version.is_some() && latest_version != Some(self.current_version.clone()) {
            self.last_result = UpdateResult::UpdateAvailable;
            self.latest_version = latest_version;
        } else {
            self.last_result = UpdateResult::Ok;
        }
    }

    /// Record a failed check with exponential backoff
    pub fn record_failure(&mut self, reason: &str, config: &UpdateConfig) {
        self.last_check_epoch = Self::now_epoch();
        self.last_result = UpdateResult::Failed;
        self.last_failure_reason = Some(reason.to_string());
        self.consecutive_failures += 1;

        // Exponential backoff: interval * 2^failures, capped at max_backoff
        let backoff = config.interval_seconds * (1 << self.consecutive_failures.min(10));
        let capped_backoff = backoff.min(config.max_backoff_seconds);
        self.next_check_epoch = Self::now_epoch() + capped_backoff;
    }

    /// Schedule next check based on config interval
    pub fn schedule_next(&mut self, config: &UpdateConfig) {
        self.next_check_epoch = Self::now_epoch() + config.interval_seconds;
    }

    /// Check if an update check is due
    pub fn is_check_due(&self) -> bool {
        self.next_check_epoch > 0 && Self::now_epoch() >= self.next_check_epoch
    }

    /// Format last check time for display (v7.29.0: real timestamp)
    pub fn format_last_check(&self) -> String {
        if self.last_check_epoch == 0 {
            return "never".to_string();
        }
        format_epoch_time(self.last_check_epoch)
    }

    /// Format next check time for display (v7.29.0: real timestamp)
    pub fn format_next_check(&self) -> String {
        if self.next_check_epoch == 0 {
            return "not scheduled".to_string();
        }
        let now = Self::now_epoch();
        if self.next_check_epoch <= now {
            return "now".to_string();
        }
        let secs = self.next_check_epoch - now;
        if secs < 60 {
            format!("in {}s", secs)
        } else if secs < 3600 {
            format!("in {}m", secs / 60)
        } else {
            format!("in {}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    /// Format last result for display
    pub fn format_last_result(&self) -> String {
        match &self.last_result {
            UpdateResult::Pending => "pending".to_string(),
            UpdateResult::Ok => "ok".to_string(),
            UpdateResult::UpdateAvailable => {
                if let Some(ref ver) = self.latest_version {
                    format!("update available: {}", ver)
                } else {
                    "update available".to_string()
                }
            }
            UpdateResult::Failed => {
                if let Some(ref reason) = self.last_failure_reason {
                    format!("failed: {}", reason)
                } else {
                    "failed".to_string()
                }
            }
        }
    }
}

/// Format epoch time as human-readable timestamp
fn format_epoch_time(epoch: u64) -> String {
    use chrono::{DateTime, Local, TimeZone};
    match Local.timestamp_opt(epoch as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => format!("epoch:{}", epoch),
    }
}

/// Complete Anna configuration v7.29.0
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

    #[serde(default)]
    pub cache: CacheConfig,

    #[serde(default)]
    pub instrumentation: InstrumentationSettings,
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
        assert_eq!(config.telemetry.sample_interval_secs, 30);
        assert_eq!(config.telemetry.log_scan_interval_secs, 60);
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.max_keys, 5000);
        // v7.26.0: instrumentation defaults
        assert!(config.instrumentation.auto_install_enabled);
        assert!(!config.instrumentation.allow_aur);
        assert_eq!(config.instrumentation.max_installs_per_day, 1);
    }

    #[test]
    fn test_telemetry_clamping() {
        // Test sample_interval clamping
        let mut settings = TelemetrySettings {
            sample_interval_secs: 1,
            ..Default::default()
        };
        assert_eq!(settings.effective_sample_interval(), 5);
        assert!(settings.sample_interval_was_clamped());

        settings.sample_interval_secs = 500;
        assert_eq!(settings.effective_sample_interval(), 300);
        assert!(settings.sample_interval_was_clamped());

        settings.sample_interval_secs = 30;
        assert_eq!(settings.effective_sample_interval(), 30);
        assert!(!settings.sample_interval_was_clamped());

        // Test max_keys clamping
        settings.max_keys = 10;
        assert_eq!(settings.effective_max_keys(), 100);
        assert!(settings.max_keys_was_clamped());

        settings.max_keys = 100000;
        assert_eq!(settings.effective_max_keys(), 50000);
        assert!(settings.max_keys_was_clamped());
    }

    #[test]
    fn test_toml_serialization() {
        let config = AnnaConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[core]"));
        assert!(toml_str.contains("[telemetry]"));
        assert!(toml_str.contains("[instrumentation]"));
    }

    #[test]
    fn test_instrumentation_aur_gate_default_off() {
        // v7.26.0: AUR gate must be OFF by default
        let config = AnnaConfig::default();
        assert!(!config.instrumentation.allow_aur, "AUR gate must be OFF by default");
    }
}
