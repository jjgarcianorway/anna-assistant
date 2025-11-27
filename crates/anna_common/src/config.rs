//! Anna Configuration Schema v0.8.0
//!
//! Natural language configuration via annactl.
//! All configuration is manipulated through natural language prompts.
//! v0.8.0: Added logging configuration.

use crate::logging::LogConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Minimum allowed update interval in seconds (10 minutes)
pub const MIN_UPDATE_INTERVAL: u64 = 600;

/// Default paths for config
const SYSTEM_CONFIG_DIR: &str = "/etc/anna";
const USER_CONFIG_SUBDIR: &str = ".config/anna";
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_preferred_model")]
    pub preferred_model: String,
    #[serde(default = "default_fallback_model")]
    pub fallback_model: String,
    #[serde(default)]
    pub selection_mode: LlmSelectionMode,
}

fn default_preferred_model() -> String {
    "llama3.2:3b".to_string()
}

fn default_fallback_model() -> String {
    "llama3.2:3b".to_string()
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
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
    86400 // 24 hours
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

/// Complete Anna configuration schema v0.8.0
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
}

impl AnnaConfigV5 {
    /// Load config from disk (user config overrides system config)
    pub fn load() -> Self {
        // Try user config first
        if let Some(user_path) = user_config_path() {
            if user_path.exists() {
                if let Ok(content) = fs::read_to_string(&user_path) {
                    if let Ok(config) = toml::from_str(&content) {
                        return config;
                    }
                }
            }
        }

        // Try system config
        let system_path = system_config_path();
        if system_path.exists() {
            if let Ok(content) = fs::read_to_string(&system_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Save config to user config file
    pub fn save(&self) -> std::io::Result<()> {
        let path = user_config_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "No home directory")
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(path, content)
    }

    /// Check if dev auto-update is active
    pub fn is_dev_auto_update_active(&self) -> bool {
        self.core.mode == CoreMode::Dev && self.update.enabled
    }

    /// Get the active model based on selection mode
    pub fn active_model(&self) -> &str {
        &self.llm.preferred_model
    }
}

/// Get user config path
pub fn user_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(USER_CONFIG_SUBDIR).join(CONFIG_FILE))
}

/// Get system config path
pub fn system_config_path() -> PathBuf {
    PathBuf::from(SYSTEM_CONFIG_DIR).join(CONFIG_FILE)
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
        assert_eq!(config.preferred_model, "llama3.2:3b");
    }

    #[test]
    fn test_update_settings_default() {
        let settings = UpdateSettings::default();
        assert!(settings.enabled); // Auto-update enabled by default
        assert_eq!(settings.interval_seconds, 86400);
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
