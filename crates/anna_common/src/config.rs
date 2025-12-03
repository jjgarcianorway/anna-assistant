//! Anna Configuration v0.0.22 - Reliability Engineering
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

/// Update channel (v0.0.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    #[default]
    Stable,
    Canary,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Canary => "canary",
        }
    }
}

/// Auto-update configuration (v0.0.11 enhanced with channels)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Update mode: auto or manual (v7.29.0)
    #[serde(default)]
    pub mode: UpdateMode,

    /// Update channel: stable or canary (v0.0.11)
    #[serde(default)]
    pub channel: UpdateChannel,

    /// Check interval in seconds (default 600 = 10 minutes)
    #[serde(default = "default_update_interval_seconds")]
    pub interval_seconds: u64,

    /// Maximum backoff on failure (6 hours)
    #[serde(default = "default_max_backoff_seconds")]
    pub max_backoff_seconds: u64,

    /// Minimum disk space required for update in bytes (default 100 MB)
    #[serde(default = "default_min_disk_space")]
    pub min_disk_space_bytes: u64,
}

fn default_min_disk_space() -> u64 {
    100 * 1024 * 1024 // 100 MB
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
            channel: UpdateChannel::Stable,
            interval_seconds: default_update_interval_seconds(),
            max_backoff_seconds: default_max_backoff_seconds(),
            min_disk_space_bytes: default_min_disk_space(),
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

/// Update check result (v0.0.11 enhanced)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdateResult {
    #[default]
    Pending,
    Ok,
    UpdateAvailable,
    Failed,
    /// v0.0.11: Update in progress
    InProgress,
    /// v0.0.11: Update completed successfully
    UpdatedTo,
    /// v0.0.11: Update was rolled back
    RolledBack,
}

impl UpdateResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateResult::Pending => "pending",
            UpdateResult::Ok => "ok",
            UpdateResult::UpdateAvailable => "update available",
            UpdateResult::Failed => "failed",
            UpdateResult::InProgress => "in progress",
            UpdateResult::UpdatedTo => "updated",
            UpdateResult::RolledBack => "rolled back",
        }
    }
}

/// Auto-update state (v0.0.11 enhanced, stored in /var/lib/anna/internal/update_state.json)
///
/// v0.0.11: Added channel, update phase, and progress tracking
/// v7.34.0: Full spec compliance with real timestamps and audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    /// Mode: auto, manual, or off
    #[serde(default)]
    pub mode: UpdateMode,
    /// Channel: stable or canary (v0.0.11)
    #[serde(default)]
    pub channel: UpdateChannel,
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
    /// v0.0.11: Current update phase (if update in progress)
    #[serde(default)]
    pub update_phase: Option<String>,
    /// v0.0.11: Update progress percentage (0-100)
    #[serde(default)]
    pub update_progress_percent: Option<u8>,
    /// v0.0.11: Estimated time remaining for update (seconds)
    #[serde(default)]
    pub update_eta_seconds: Option<u64>,
    /// v0.0.11: Version being updated to
    #[serde(default)]
    pub updating_to_version: Option<String>,
    /// v0.0.11: Last successful update timestamp
    #[serde(default)]
    pub last_update_at: Option<u64>,
    /// v0.0.11: Previous version (for rollback info)
    #[serde(default)]
    pub previous_version: Option<String>,
}

fn default_interval_seconds() -> u64 {
    600 // 10 minutes
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            mode: UpdateMode::Auto,
            channel: UpdateChannel::Stable,
            interval_seconds: 600,
            last_check_at: 0,
            last_result: UpdateResult::Pending,
            last_error: None,
            last_checked_version_installed: String::new(),
            last_checked_version_available: None,
            next_check_at: 0,
            last_successful_check_at: None,
            consecutive_failures: 0,
            update_phase: None,
            update_progress_percent: None,
            update_eta_seconds: None,
            updating_to_version: None,
            last_update_at: None,
            previous_version: None,
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
    /// v0.0.27: Clear stale "update available" if current version is >= available
    pub fn initialize_on_start(&mut self) {
        // v0.0.27: Get current version and clear stale update info
        let current = env!("CARGO_PKG_VERSION");
        if let Some(ref available) = self.last_checked_version_available {
            // If we're running a version >= the "available" version, clear it
            if !crate::is_newer_version(available, current) {
                self.last_checked_version_available = None;
                self.last_result = UpdateResult::Ok;
                self.last_checked_version_installed = current.to_string();
            }
        }

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

    /// Format last result for display (v0.0.11 enhanced)
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
            UpdateResult::InProgress => {
                if let Some(ref phase) = self.update_phase {
                    if let Some(percent) = self.update_progress_percent {
                        format!("in progress: {} ({}%)", phase, percent)
                    } else {
                        format!("in progress: {}", phase)
                    }
                } else {
                    "in progress".to_string()
                }
            }
            UpdateResult::UpdatedTo => {
                if let Some(ref ver) = self.updating_to_version {
                    format!("updated to {}", ver)
                } else {
                    "updated".to_string()
                }
            }
            UpdateResult::RolledBack => {
                if let Some(ref prev) = self.previous_version {
                    format!("rolled back to {}", prev)
                } else {
                    "rolled back".to_string()
                }
            }
        }
    }

    /// v0.0.11: Set update in progress
    pub fn set_update_in_progress(&mut self, version: &str, phase: &str, progress: Option<u8>, eta: Option<u64>) {
        self.last_result = UpdateResult::InProgress;
        self.updating_to_version = Some(version.to_string());
        self.update_phase = Some(phase.to_string());
        self.update_progress_percent = progress;
        self.update_eta_seconds = eta;
    }

    /// v0.0.11: Record successful update
    pub fn record_update_success(&mut self, new_version: &str, previous_version: &str) {
        let now = Self::now_epoch();
        self.last_result = UpdateResult::UpdatedTo;
        self.updating_to_version = Some(new_version.to_string());
        self.previous_version = Some(previous_version.to_string());
        self.last_update_at = Some(now);
        self.update_phase = None;
        self.update_progress_percent = None;
        self.update_eta_seconds = None;
        self.last_checked_version_installed = new_version.to_string();
        self.last_checked_version_available = None;
        self.consecutive_failures = 0;
    }

    /// v0.0.11: Record rollback
    pub fn record_rollback(&mut self, rolled_back_to: &str, reason: &str) {
        self.last_result = UpdateResult::RolledBack;
        self.previous_version = Some(rolled_back_to.to_string());
        self.last_error = Some(reason.to_string());
        self.update_phase = None;
        self.update_progress_percent = None;
        self.update_eta_seconds = None;
        self.updating_to_version = None;
    }

    /// v0.0.11: Clear update progress (after completion or failure)
    pub fn clear_update_progress(&mut self) {
        self.update_phase = None;
        self.update_progress_percent = None;
        self.update_eta_seconds = None;
    }

    /// v0.0.11: Format channel for display
    pub fn format_channel(&self) -> &'static str {
        self.channel.as_str()
    }

    /// v0.0.11: Check if update is in progress
    pub fn is_update_in_progress(&self) -> bool {
        self.last_result == UpdateResult::InProgress
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

/// LLM settings (v0.0.5)
/// Controls local LLM via Ollama for both Translator and Junior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    /// Whether LLM features are enabled (default: true)
    #[serde(default = "default_llm_enabled")]
    pub enabled: bool,

    /// Ollama API URL (default: http://127.0.0.1:11434)
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// Translator settings
    #[serde(default)]
    pub translator: RoleSettings,

    /// Junior settings
    #[serde(default)]
    pub junior: RoleSettings,

    /// Candidate models for Translator (empty = use defaults)
    #[serde(default)]
    pub translator_candidates: Vec<String>,

    /// Candidate models for Junior (empty = use defaults)
    #[serde(default)]
    pub junior_candidates: Vec<String>,
}

/// Settings for a specific LLM role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSettings {
    /// Model to use (auto-selected if empty)
    #[serde(default)]
    pub model: String,

    /// Timeout for LLM generation in milliseconds
    #[serde(default = "default_role_timeout_ms")]
    pub timeout_ms: u64,

    /// Whether this role is enabled
    #[serde(default = "default_role_enabled")]
    pub enabled: bool,
}

fn default_llm_enabled() -> bool {
    true
}

fn default_role_enabled() -> bool {
    true
}

fn default_role_timeout_ms() -> u64 {
    60000 // 60 seconds
}

fn default_ollama_url() -> String {
    "http://127.0.0.1:11434".to_string()
}

impl Default for RoleSettings {
    fn default() -> Self {
        Self {
            model: String::new(),
            timeout_ms: default_role_timeout_ms(),
            enabled: default_role_enabled(),
        }
    }
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            enabled: default_llm_enabled(),
            ollama_url: default_ollama_url(),
            translator: RoleSettings {
                timeout_ms: 10000, // 10s for fast translator
                ..Default::default()
            },
            junior: RoleSettings {
                timeout_ms: 60000, // 60s for junior
                ..Default::default()
            },
            translator_candidates: Vec::new(),
            junior_candidates: Vec::new(),
        }
    }
}

impl LlmSettings {
    /// Get effective translator model (for display)
    pub fn effective_translator_model(&self) -> &str {
        if self.translator.model.is_empty() {
            "(auto-select)"
        } else {
            &self.translator.model
        }
    }

    /// Get effective junior model (for display)
    pub fn effective_junior_model(&self) -> &str {
        if self.junior.model.is_empty() {
            "(auto-select)"
        } else {
            &self.junior.model
        }
    }
}

/// Legacy Junior settings (v0.0.4 compatibility)
/// Deprecated: use LlmSettings instead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorSettings {
    #[serde(default = "default_role_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_role_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
}

impl Default for JuniorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            model: String::new(),
            timeout_ms: 60000,
            ollama_url: default_ollama_url(),
        }
    }
}

impl JuniorSettings {
    pub fn effective_model(&self) -> &str {
        if self.model.is_empty() {
            "(auto-select)"
        } else {
            &self.model
        }
    }
}

/// Junior state (stored in /var/lib/anna/internal/junior_state.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JuniorState {
    /// Whether Ollama is available
    pub ollama_available: bool,
    /// Selected model name
    pub selected_model: Option<String>,
    /// Whether the selected model is downloaded
    pub model_ready: bool,
    /// Available models list
    pub available_models: Vec<String>,
    /// Last check timestamp (epoch seconds)
    pub last_check: u64,
    /// Last error message
    pub last_error: Option<String>,
}

impl JuniorState {
    /// State file path
    pub fn state_path() -> std::path::PathBuf {
        std::path::PathBuf::from(DATA_DIR).join("internal/junior_state.json")
    }

    /// Load state from file
    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    /// Save state to file
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let path_str = path.to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        crate::atomic_write(path_str, &content)
    }

    /// Check if Junior is ready for use
    pub fn is_ready(&self) -> bool {
        self.ollama_available && self.model_ready && self.selected_model.is_some()
    }

    /// Format status for display
    pub fn format_status(&self) -> String {
        if !self.ollama_available {
            return "Ollama not available".to_string();
        }
        if !self.model_ready {
            if let Some(ref model) = self.selected_model {
                return format!("Model '{}' not downloaded", model);
            }
            return "No model selected".to_string();
        }
        if let Some(ref model) = self.selected_model {
            format!("Ready ({})", model)
        } else {
            "Ready (no model)".to_string()
        }
    }
}

/// Complete Anna configuration v0.0.5
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

    /// LLM settings (v0.0.5) - role-based model selection
    #[serde(default)]
    pub llm: LlmSettings,

    /// Junior verifier settings (v0.0.4 - deprecated, use llm.junior)
    #[serde(default)]
    pub junior: JuniorSettings,

    /// Memory and learning settings (v0.0.13)
    #[serde(default)]
    pub memory: MemoryConfig,

    /// UI settings (v0.0.15)
    #[serde(default)]
    pub ui: UiConfig,

    /// Performance settings (v0.0.21)
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// Reliability settings (v0.0.22)
    #[serde(default)]
    pub reliability: ReliabilityConfig,
}

/// UI configuration (v0.0.15)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Debug level: 0=minimal, 1=normal (default), 2=full
    /// 0: Only [you]->[anna] and final [anna]->[you], plus confirmations
    /// 1: Dialogues condensed, tool calls summarized, evidence IDs included
    /// 2: Full dialogues, tool execution summaries, Junior critique in full
    #[serde(default = "default_debug_level")]
    pub debug_level: u8,

    /// Whether to use colors in output (true color if available)
    #[serde(default = "default_colors_enabled")]
    pub colors_enabled: bool,

    /// Maximum width for text wrapping (0 = auto-detect terminal width)
    #[serde(default)]
    pub max_width: u16,
}

/// Performance configuration (v0.0.21)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Token budget settings per role
    #[serde(default)]
    pub budgets: PerformanceBudgets,

    /// Cache settings
    #[serde(default)]
    pub cache: PerformanceCacheConfig,
}

/// Token budget settings per role (v0.0.21)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudgets {
    /// Translator max tokens (default: 256)
    #[serde(default = "default_translator_max_tokens")]
    pub translator_max_tokens: u32,

    /// Translator max time in ms (default: 1500)
    #[serde(default = "default_translator_max_ms")]
    pub translator_max_ms: u64,

    /// Junior max tokens (default: 384)
    #[serde(default = "default_junior_max_tokens")]
    pub junior_max_tokens: u32,

    /// Junior max time in ms (default: 2500)
    #[serde(default = "default_junior_max_ms")]
    pub junior_max_ms: u64,

    /// Whether to log budget overruns
    #[serde(default = "default_true_fn")]
    pub log_overruns: bool,
}

fn default_translator_max_tokens() -> u32 { 256 }
fn default_translator_max_ms() -> u64 { 1500 }
fn default_junior_max_tokens() -> u32 { 384 }
fn default_junior_max_ms() -> u64 { 2500 }
fn default_true_fn() -> bool { true }

impl Default for PerformanceBudgets {
    fn default() -> Self {
        Self {
            translator_max_tokens: default_translator_max_tokens(),
            translator_max_ms: default_translator_max_ms(),
            junior_max_tokens: default_junior_max_tokens(),
            junior_max_ms: default_junior_max_ms(),
            log_overruns: true,
        }
    }
}

/// Cache configuration (v0.0.21)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCacheConfig {
    /// Enable tool result caching
    #[serde(default = "default_true_fn")]
    pub tool_cache_enabled: bool,

    /// Tool cache TTL in seconds (default: 300 = 5 min)
    #[serde(default = "default_tool_cache_ttl")]
    pub tool_cache_ttl_secs: u64,

    /// Enable LLM response caching
    #[serde(default = "default_true_fn")]
    pub llm_cache_enabled: bool,

    /// LLM cache TTL in seconds (default: 600 = 10 min)
    #[serde(default = "default_llm_cache_ttl")]
    pub llm_cache_ttl_secs: u64,

    /// Maximum cache entries per category
    #[serde(default = "default_max_cache_entries")]
    pub max_entries: usize,
}

fn default_tool_cache_ttl() -> u64 { 300 }
fn default_llm_cache_ttl() -> u64 { 600 }
fn default_max_cache_entries() -> usize { 1000 }

impl Default for PerformanceCacheConfig {
    fn default() -> Self {
        Self {
            tool_cache_enabled: true,
            tool_cache_ttl_secs: default_tool_cache_ttl(),
            llm_cache_enabled: true,
            llm_cache_ttl_secs: default_llm_cache_ttl(),
            max_entries: default_max_cache_entries(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            budgets: PerformanceBudgets::default(),
            cache: PerformanceCacheConfig::default(),
        }
    }
}

/// Reliability configuration (v0.0.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    /// Error budget thresholds
    #[serde(default)]
    pub error_budgets: ReliabilityBudgets,

    /// Metrics settings
    #[serde(default)]
    pub metrics: MetricsSettings,
}

/// Error budget thresholds (v0.0.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityBudgets {
    /// Max request failure rate (percentage per day, default: 1.0)
    #[serde(default = "default_request_failure")]
    pub request_failure_percent: f64,

    /// Max tool failure rate (percentage per day, default: 2.0)
    #[serde(default = "default_tool_failure")]
    pub tool_failure_percent: f64,

    /// Max mutation rollback rate (percentage per day, default: 0.5)
    #[serde(default = "default_mutation_rollback")]
    pub mutation_rollback_percent: f64,

    /// Max LLM timeout rate (percentage per day, default: 3.0)
    #[serde(default = "default_llm_timeout")]
    pub llm_timeout_percent: f64,
}

fn default_request_failure() -> f64 { 1.0 }
fn default_tool_failure() -> f64 { 2.0 }
fn default_mutation_rollback() -> f64 { 0.5 }
fn default_llm_timeout() -> f64 { 3.0 }

impl Default for ReliabilityBudgets {
    fn default() -> Self {
        Self {
            request_failure_percent: default_request_failure(),
            tool_failure_percent: default_tool_failure(),
            mutation_rollback_percent: default_mutation_rollback(),
            llm_timeout_percent: default_llm_timeout(),
        }
    }
}

/// Metrics collection settings (v0.0.22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSettings {
    /// Enable metrics collection
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,

    /// Retention days for metrics (default: 7)
    #[serde(default = "default_metrics_retention")]
    pub retention_days: u32,
}

fn default_metrics_enabled() -> bool { true }
fn default_metrics_retention() -> u32 { 7 }

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            retention_days: default_metrics_retention(),
        }
    }
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            error_budgets: ReliabilityBudgets::default(),
            metrics: MetricsSettings::default(),
        }
    }
}

fn default_debug_level() -> u8 { 1 }
fn default_colors_enabled() -> bool { true }

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            debug_level: default_debug_level(),
            colors_enabled: default_colors_enabled(),
            max_width: 0,
        }
    }
}

impl UiConfig {
    /// Get the effective max width (auto-detect if 0)
    pub fn effective_width(&self) -> u16 {
        if self.max_width > 0 {
            return self.max_width;
        }
        // Auto-detect terminal width
        if let Some((width, _)) = terminal_size::terminal_size() {
            width.0.min(120) // Cap at 120 for readability
        } else {
            80 // Default fallback
        }
    }

    /// Check if debug level shows full dialogues
    pub fn is_full_debug(&self) -> bool {
        self.debug_level >= 2
    }

    /// Check if debug level shows condensed dialogues
    pub fn is_normal_debug(&self) -> bool {
        self.debug_level >= 1
    }

    /// Check if debug level is minimal (user-facing only)
    pub fn is_minimal(&self) -> bool {
        self.debug_level == 0
    }

    /// Format debug level for display
    pub fn format_debug_level(&self) -> &'static str {
        match self.debug_level {
            0 => "minimal",
            1 => "normal",
            _ => "full",
        }
    }
}

/// Memory and learning configuration (v0.0.13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Whether memory/learning is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to store raw transcripts (privacy: default false)
    #[serde(default)]
    pub store_raw: bool,

    /// Maximum sessions to keep in memory index
    #[serde(default = "default_max_sessions")]
    pub max_sessions: u64,

    /// Minimum reliability score to create a recipe (0-100)
    #[serde(default = "default_min_reliability")]
    pub min_reliability_for_recipe: u32,

    /// Maximum recipes to store
    #[serde(default = "default_max_recipes")]
    pub max_recipes: u64,
}

fn default_true() -> bool { true }
fn default_max_sessions() -> u64 { 10000 }
fn default_min_reliability() -> u32 { 80 }
fn default_max_recipes() -> u64 { 500 }

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            store_raw: false,
            max_sessions: default_max_sessions(),
            min_reliability_for_recipe: default_min_reliability(),
            max_recipes: default_max_recipes(),
        }
    }
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

    // v0.0.15: UI config tests
    #[test]
    fn test_ui_config_defaults() {
        let config = AnnaConfig::default();
        assert_eq!(config.ui.debug_level, 1, "Default debug level should be 1 (normal)");
        assert!(config.ui.colors_enabled, "Colors should be enabled by default");
        assert_eq!(config.ui.max_width, 0, "Default max_width should be 0 (auto-detect)");
    }

    #[test]
    fn test_ui_config_debug_level_helpers() {
        let mut config = UiConfig::default();

        // Level 0 = minimal
        config.debug_level = 0;
        assert!(config.is_minimal());
        assert!(!config.is_normal_debug());
        assert!(!config.is_full_debug());
        assert_eq!(config.format_debug_level(), "minimal");

        // Level 1 = normal
        config.debug_level = 1;
        assert!(!config.is_minimal());
        assert!(config.is_normal_debug());
        assert!(!config.is_full_debug());
        assert_eq!(config.format_debug_level(), "normal");

        // Level 2 = full
        config.debug_level = 2;
        assert!(!config.is_minimal());
        assert!(config.is_normal_debug());
        assert!(config.is_full_debug());
        assert_eq!(config.format_debug_level(), "full");
    }

    #[test]
    fn test_ui_config_effective_width() {
        let mut config = UiConfig::default();

        // 0 means auto-detect (should return > 0)
        config.max_width = 0;
        assert!(config.effective_width() > 0);

        // Explicit width should be returned as-is
        config.max_width = 120;
        assert_eq!(config.effective_width(), 120);
    }

    #[test]
    fn test_ui_config_toml_parsing() {
        // Test parsing UiConfig fields directly (without [ui] section header)
        let toml_str = r#"
debug_level = 2
colors_enabled = false
max_width = 100
"#;
        let config: UiConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.debug_level, 2);
        assert!(!config.colors_enabled);
        assert_eq!(config.max_width, 100);
    }

    #[test]
    fn test_ui_config_in_anna_config() {
        // Test parsing UI config within full AnnaConfig structure
        let toml_str = r#"
[core]
mode = "normal"

[ui]
debug_level = 0
colors_enabled = false
max_width = 80
"#;
        let config: AnnaConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ui.debug_level, 0);
        assert!(!config.ui.colors_enabled);
        assert_eq!(config.ui.max_width, 80);
    }
}
