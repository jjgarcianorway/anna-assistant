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

/// Auto-update state (v7.34.0 spec-compliant, stored in /var/lib/anna/internal/update_state.json)
///
/// v7.34.0: Full spec compliance with real timestamps and audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    /// Mode: auto, manual, or off
    #[serde(default)]
    pub mode: UpdateMode,
    /// Check interval in seconds (default 600 = 10 minutes)
    #[serde(default = "default_interval_seconds")]
    pub interval_seconds: u64,
    /// Last check timestamp (unix epoch seconds, 0 = never)
    #[serde(default)]
    pub last_check_at: u64,
    /// Last check result: up_to_date, update_available, error
    #[serde(default)]
    pub last_result: UpdateResult,
    /// Last error message (only when last_result = error)
    #[serde(default)]
    pub last_error: Option<String>,
    /// Installed version at last check
    #[serde(default)]
    pub last_checked_version_installed: String,
    /// Available version at last check (if update available)
    #[serde(default)]
    pub last_checked_version_available: Option<String>,
    /// Next scheduled check (unix epoch seconds)
    #[serde(default)]
    pub next_check_at: u64,
    /// Last successful check timestamp (only when not error)
    #[serde(default)]
    pub last_successful_check_at: Option<u64>,
    /// Current consecutive failure count (for backoff)
    #[serde(default)]
    pub consecutive_failures: u32,
}

fn default_interval_seconds() -> u64 {
    600 // 10 minutes
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            mode: UpdateMode::Auto,
            interval_seconds: 600,
            last_check_at: 0,
            last_result: UpdateResult::Pending,
            last_error: None,
            last_checked_version_installed: String::new(),
            last_checked_version_available: None,
            next_check_at: 0,
            last_successful_check_at: None,
            consecutive_failures: 0,
        }
    }
}

impl UpdateState {
    /// State file path (v7.34.0: canonical path)
    pub fn state_path() -> PathBuf {
        PathBuf::from(DATA_DIR).join("internal/update_state.json")
    }

    /// Ensure the internal directory exists
    pub fn ensure_internal_dir() -> std::io::Result<()> {
        let internal_dir = PathBuf::from(DATA_DIR).join("internal");
        fs::create_dir_all(&internal_dir)
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
        Self::ensure_internal_dir()?;
        let path = Self::state_path();
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

    /// Check if an update check is due (v7.34.0)
    pub fn is_check_due(&self) -> bool {
        // Never check if mode is not Auto
        if self.mode != UpdateMode::Auto {
            return false;
        }
        // If never checked, check now
        if self.last_check_at == 0 {
            return true;
        }
        // Check if next_check_at has passed
        if self.next_check_at > 0 {
            return Self::now_epoch() >= self.next_check_at;
        }
        // Fallback: check if interval has passed since last check
        Self::now_epoch() >= self.last_check_at + self.interval_seconds
    }

    /// Record a successful check (v7.34.0)
    pub fn record_success(&mut self, current_version: &str, latest_version: Option<String>) {
        let now = Self::now_epoch();
        self.last_check_at = now;
        self.last_checked_version_installed = current_version.to_string();
        self.last_error = None;
        self.consecutive_failures = 0;
        self.last_successful_check_at = Some(now);

        if let Some(ref ver) = latest_version {
            if ver != current_version {
                self.last_result = UpdateResult::UpdateAvailable;
                self.last_checked_version_available = Some(ver.clone());
            } else {
                self.last_result = UpdateResult::Ok;
                self.last_checked_version_available = None;
            }
        } else {
            self.last_result = UpdateResult::Ok;
            self.last_checked_version_available = None;
        }

        // Schedule next check
        self.next_check_at = now + self.interval_seconds;
    }

    /// Record a failed check with exponential backoff (v7.34.0)
    pub fn record_failure(&mut self, current_version: &str, error: &str) {
        let now = Self::now_epoch();
        self.last_check_at = now;
        self.last_checked_version_installed = current_version.to_string();
        self.last_result = UpdateResult::Failed;
        self.last_error = Some(error.to_string());
        self.consecutive_failures += 1;

        // Exponential backoff: interval * 2^failures, capped at 1 hour
        let backoff_multiplier = 1u64 << self.consecutive_failures.min(6);
        let backoff = self.interval_seconds * backoff_multiplier;
        let capped_backoff = backoff.min(3600); // Max 1 hour
        self.next_check_at = now + capped_backoff;
    }

    /// Initialize on daemon start (v7.34.0)
    /// If never checked, schedule first check within 60 seconds
    pub fn initialize_on_start(&mut self) {
        if self.last_check_at == 0 && self.mode == UpdateMode::Auto {
            // Never checked - schedule within 60 seconds
            self.next_check_at = Self::now_epoch() + 60;
        } else if self.next_check_at == 0 && self.mode == UpdateMode::Auto {
            // No next check scheduled - schedule based on last check
            self.next_check_at = self.last_check_at + self.interval_seconds;
        }
    }

    /// Format last check time for display (v7.34.0: real timestamp or "never")
    pub fn format_last_check(&self) -> String {
        if self.last_check_at == 0 {
            return "never".to_string();
        }
        format_epoch_time(self.last_check_at)
    }

    /// Format next check time for display (v7.34.0)
    pub fn format_next_check(&self) -> String {
        if self.mode != UpdateMode::Auto {
            return "n/a (not in auto mode)".to_string();
        }
        if self.next_check_at == 0 {
            return "not scheduled".to_string();
        }
        let now = Self::now_epoch();
        if self.next_check_at <= now {
            return "now".to_string();
        }
        let secs = self.next_check_at - now;
        if secs < 60 {
            format!("in {}s", secs)
        } else if secs < 3600 {
            format!("in {}m", secs / 60)
        } else {
            format!("in {}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    /// Format last result for display (v7.34.0)
    pub fn format_last_result(&self) -> String {
        match &self.last_result {
            UpdateResult::Pending => "pending".to_string(),
            UpdateResult::Ok => "up to date".to_string(),
            UpdateResult::UpdateAvailable => {
                if let Some(ref ver) = self.last_checked_version_available {
                    format!("{} -> {}", self.last_checked_version_installed, ver)
                } else {
                    "update available".to_string()
                }
            }
            UpdateResult::Failed => {
                if let Some(ref error) = self.last_error {
                    format!("error: {}", error)
                } else {
                    "error".to_string()
                }
            }
        }
    }

    /// Format mode for display
    pub fn format_mode(&self) -> &'static str {
        match self.mode {
            UpdateMode::Auto => "auto",
            UpdateMode::Manual => "manual",
        }
    }

    /// Format interval for display
    pub fn format_interval(&self) -> String {
        let secs = self.interval_seconds;
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else {
            format!("{}h", secs / 3600)
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
