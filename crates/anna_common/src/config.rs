//! Anna Configuration Schema v0.80.0
//!
//! System-wide configuration managed by administrators.
//! Configuration lives in /etc/anna/config.toml - not per-user.
//! This is intentional: the sysadmin controls the LLM model and settings,
//! not individual users.
//!
//! v0.8.0: Added logging configuration.
//! v0.15.0: Simplified to system-only config (no user override).
//! v0.80.0: Added orchestration profile (razorback-fast for fast path).

use crate::logging::LogConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Minimum allowed update interval in seconds (10 minutes)
pub const MIN_UPDATE_INTERVAL: u64 = 600;

/// System configuration directory - the ONLY place config lives
/// This is intentional: administrators control Anna settings, not users.
pub const SYSTEM_CONFIG_DIR: &str = "/etc/anna";
const CONFIG_FILE: &str = "config.toml";

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

/// LLM model selection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmSelectionMode {
    #[default]
    Auto,
    Manual,
}

impl LlmSelectionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmSelectionMode::Auto => "auto",
            LlmSelectionMode::Manual => "manual",
        }
    }
}

/// Update channel (future-proof, but currently all releases on main channel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Main,
    Stable,
    Beta,
    Dev,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Main => "main",
            Channel::Stable => "stable",
            Channel::Beta => "beta",
            Channel::Dev => "dev",
        }
    }
}

/// v0.80.0: Orchestration profile for per-host optimization
///
/// Profiles control how the Junior→Senior loop behaves:
/// - `default`: Original behavior (up to 3 iterations, full prompts)
/// - `razorback_fast`: Optimized for fast hardware (max 1 iteration, minimal prompts, 5s budget)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum OrchestrationProfile {
    #[default]
    Default,
    RazorbackFast,
}

impl OrchestrationProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrchestrationProfile::Default => "default",
            OrchestrationProfile::RazorbackFast => "razorback-fast",
        }
    }

    pub fn is_razorback_fast(&self) -> bool {
        matches!(self, OrchestrationProfile::RazorbackFast)
    }

    /// Get the max iterations for this profile
    pub fn max_iterations(&self) -> usize {
        match self {
            OrchestrationProfile::Default => 3,
            OrchestrationProfile::RazorbackFast => 1, // Single pass for simple questions
        }
    }

    /// Get the timeout budget in seconds
    pub fn timeout_budget_secs(&self) -> u64 {
        match self {
            OrchestrationProfile::Default => 60,
            OrchestrationProfile::RazorbackFast => 5, // Hard 5-second budget
        }
    }
}

/// v0.80.0: Orchestration configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// Which orchestration profile to use
    #[serde(default)]
    pub profile: OrchestrationProfile,
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            profile: OrchestrationProfile::Default,
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

/// LLM configuration section
///
/// Supports role-specific models for optimized resource usage:
/// - `junior_model`: Fast model for LLM-A (probe execution, command parsing)
/// - `senior_model`: Smarter model for LLM-B (reasoning, synthesis)
/// - `preferred_model`: Legacy single-model config (used if junior/senior not set)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Model for junior role (LLM-A) - probe executor, needs speed
    #[serde(default)]
    pub junior_model: Option<String>,
    /// Model for senior role (LLM-B) - reasoner, needs quality
    #[serde(default)]
    pub senior_model: Option<String>,
    /// Legacy: single model for all roles (backwards compatible)
    #[serde(default = "default_preferred_model")]
    pub preferred_model: String,
    #[serde(default = "default_fallback_model")]
    pub fallback_model: String,
    #[serde(default)]
    pub selection_mode: LlmSelectionMode,
}

fn default_preferred_model() -> String {
    "qwen3:8b".to_string() // v0.16.0: Qwen3 is better for JSON/agent tasks
}

fn default_fallback_model() -> String {
    "qwen3:1.7b".to_string() // v0.16.0: Fast fallback model
}

impl LlmConfig {
    /// Get the model for junior role (LLM-A)
    /// Falls back to preferred_model if junior_model not set
    pub fn get_junior_model(&self) -> &str {
        self.junior_model
            .as_deref()
            .unwrap_or(&self.preferred_model)
    }

    /// Get the model for senior role (LLM-B)
    /// Falls back to preferred_model if senior_model not set
    pub fn get_senior_model(&self) -> &str {
        self.senior_model
            .as_deref()
            .unwrap_or(&self.preferred_model)
    }

    /// Check if this is an old config without role-specific models
    pub fn needs_role_model_migration(&self) -> bool {
        self.junior_model.is_none() && self.senior_model.is_none()
    }

    /// Suggest optimal junior model based on senior model
    /// Junior needs speed, so use smaller model when senior is large
    /// v0.16.0: Updated to use Qwen3 models
    pub fn suggest_junior_model(&self) -> String {
        let senior = self.get_senior_model();

        // If senior is a very large model (70B+), use 8B for junior
        if senior.contains("70b") || senior.contains("72b") {
            "qwen3:8b".to_string()
        }
        // If senior is a large model (14B-32B), use 4B for junior
        else if senior.contains("32b") || senior.contains("30b") || senior.contains("14b") {
            "qwen3:4b".to_string()
        }
        // Default: fast 1.7B model for junior (great for routing/probes)
        else {
            "qwen3:1.7b".to_string()
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            junior_model: None,
            senior_model: None,
            preferred_model: default_preferred_model(),
            fallback_model: default_fallback_model(),
            selection_mode: LlmSelectionMode::Auto,
        }
    }
}

/// Update configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_interval_seconds")]
    pub interval_seconds: u64,
    #[serde(default)]
    pub channel: Channel,
}

fn default_interval_seconds() -> u64 {
    600 // 10 minutes - fast iteration during development
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            enabled: true, // Auto-update always enabled by default
            interval_seconds: default_interval_seconds(),
            channel: Channel::Main,
        }
    }
}

impl UpdateSettings {
    /// Get the effective interval, enforcing minimum
    pub fn effective_interval(&self) -> u64 {
        self.interval_seconds.max(MIN_UPDATE_INTERVAL)
    }
}

/// Complete Anna configuration schema v0.80.0
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnaConfigV5 {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub update: UpdateSettings,
    #[serde(default)]
    pub log: LogConfig,
    /// v0.80.0: Orchestration profile for per-host optimization
    #[serde(default)]
    pub orchestration: OrchestrationConfig,
}

impl AnnaConfigV5 {
    /// Load config from system config directory (/etc/anna/config.toml)
    /// v0.15.0: Config is system-wide only, no per-user override.
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

    /// Save config to system config file (/etc/anna/config.toml)
    /// Note: Requires root/sudo to write to /etc/anna
    pub fn save(&self) -> std::io::Result<()> {
        let path = config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(path, content)
    }

    /// Check if dev auto-update is active (for frequent 10-min checks in dev mode)
    pub fn is_dev_auto_update_active(&self) -> bool {
        self.core.mode == CoreMode::Dev && self.update.enabled
    }

    /// Check if auto-update is enabled (works in any mode)
    /// v0.14.0: Auto-update now works in both Normal and Dev modes
    pub fn is_auto_update_enabled(&self) -> bool {
        self.update.enabled
    }

    /// Get the active model based on selection mode
    pub fn active_model(&self) -> &str {
        &self.llm.preferred_model
    }
}

/// Get the config file path (/etc/anna/config.toml)
/// v0.15.0: Single system-wide config, no per-user config.
pub fn config_path() -> PathBuf {
    PathBuf::from(SYSTEM_CONFIG_DIR).join(CONFIG_FILE)
}

/// Get the config directory (/etc/anna)
pub fn config_dir() -> PathBuf {
    PathBuf::from(SYSTEM_CONFIG_DIR)
}

/// Config change for reporting to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub path: String,
    pub from: String,
    pub to: String,
}

impl ConfigChange {
    pub fn new(path: &str, from: impl ToString, to: impl ToString) -> Self {
        Self {
            path: path.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        }
    }
}

/// Config mutation result
#[derive(Debug, Clone)]
pub struct ConfigMutation {
    pub changes: Vec<ConfigChange>,
    pub summary: String,
    pub requires_confirmation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_mode_default() {
        let config = CoreConfig::default();
        assert_eq!(config.mode, CoreMode::Normal);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.selection_mode, LlmSelectionMode::Auto);
        // v0.16.0: Default models now use Qwen3 (better JSON/agent support)
        assert_eq!(config.preferred_model, "qwen3:8b");
        assert_eq!(config.fallback_model, "qwen3:1.7b");
        // Role-specific models are None by default (legacy configs)
        assert!(config.junior_model.is_none());
        assert!(config.senior_model.is_none());
    }

    #[test]
    fn test_update_settings_default() {
        let settings = UpdateSettings::default();
        assert!(settings.enabled); // Auto-update enabled by default
        assert_eq!(settings.interval_seconds, 600);
        assert_eq!(settings.channel, Channel::Main);
    }

    #[test]
    fn test_update_settings_effective_interval_enforces_minimum() {
        let settings = UpdateSettings {
            enabled: true,
            interval_seconds: 100, // Below minimum
            channel: Channel::Dev,
        };
        assert_eq!(settings.effective_interval(), MIN_UPDATE_INTERVAL);
    }

    #[test]
    fn test_anna_config_v5_default() {
        let config = AnnaConfigV5::default();
        assert_eq!(config.core.mode, CoreMode::Normal);
        assert_eq!(config.llm.selection_mode, LlmSelectionMode::Auto);
        assert!(config.update.enabled); // Auto-update enabled by default
    }

    #[test]
    fn test_is_dev_auto_update_active() {
        let mut config = AnnaConfigV5::default();
        // Default: enabled=true, mode=Normal -> not dev auto-update
        assert!(!config.is_dev_auto_update_active());

        config.core.mode = CoreMode::Dev;
        // Dev mode with enabled=true -> is dev auto-update active
        assert!(config.is_dev_auto_update_active());

        config.update.enabled = false;
        // Dev mode but disabled -> not active
        assert!(!config.is_dev_auto_update_active());

        config.update.enabled = true;
        config.core.mode = CoreMode::Normal;
        // Normal mode with enabled=true -> not dev auto-update (just normal auto-update)
        assert!(!config.is_dev_auto_update_active());
    }

    #[test]
    fn test_channel_as_str() {
        assert_eq!(Channel::Main.as_str(), "main");
        assert_eq!(Channel::Stable.as_str(), "stable");
        assert_eq!(Channel::Beta.as_str(), "beta");
        assert_eq!(Channel::Dev.as_str(), "dev");
    }

    #[test]
    fn test_config_change() {
        let change = ConfigChange::new("update.enabled", false, true);
        assert_eq!(change.path, "update.enabled");
        assert_eq!(change.from, "false");
        assert_eq!(change.to, "true");
    }

    #[test]
    fn test_toml_serialization() {
        let config = AnnaConfigV5::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[core]"));
        assert!(toml_str.contains("[llm]"));
        assert!(toml_str.contains("[update]"));
        assert!(toml_str.contains("[log]"));
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
[core]
mode = "dev"

[llm]
preferred_model = "qwen2.5:14b"
fallback_model = "llama3.2:3b"
selection_mode = "manual"

[update]
enabled = true
interval_seconds = 600
channel = "dev"
"#;
        let config: AnnaConfigV5 = toml::from_str(toml_str).unwrap();
        assert_eq!(config.core.mode, CoreMode::Dev);
        assert_eq!(config.llm.preferred_model, "qwen2.5:14b");
        assert_eq!(config.llm.selection_mode, LlmSelectionMode::Manual);
        assert!(config.update.enabled);
        assert_eq!(config.update.interval_seconds, 600);
        assert_eq!(config.update.channel, Channel::Dev);
    }
}
