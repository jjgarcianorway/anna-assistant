//! Configuration management for Anna Assistant
//!
//! Handles loading, saving, and validating user configuration from ~/.config/anna/config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{AutonomyTier, Priority, RiskLevel};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Autonomy and auto-apply settings
    #[serde(default)]
    pub autonomy: AutonomyConfig,

    /// Notification preferences
    #[serde(default)]
    pub notifications: NotificationConfig,

    /// Snapshot and rollback settings
    #[serde(default)]
    pub snapshots: SnapshotConfig,

    /// Learning and behavior tracking
    #[serde(default)]
    pub learning: LearningConfig,

    /// Category filters and priorities
    #[serde(default)]
    pub categories: CategoryConfig,

    /// User profiles for multi-user systems
    #[serde(default)]
    pub profiles: Vec<UserProfile>,

    /// LLM configuration (v6.54.1)
    #[serde(default)]
    pub llm: LlmUserConfig,
}

impl Config {
    /// Get the default config file path: ~/.config/anna/config.toml
    pub fn default_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let config_dir = Path::new(&home).join(".config").join("anna");
        Ok(config_dir.join("config.toml"))
    }

    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path).context("Failed to read config file")?;
            let config: Config =
                toml::from_str(&contents).context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let toml_string = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&path, toml_string).context("Failed to write config file")?;

        Ok(())
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        // Check refresh interval
        if self.general.refresh_interval_seconds < 60 {
            anyhow::bail!("Refresh interval must be at least 60 seconds");
        }

        // Check snapshot retention
        if self.snapshots.max_snapshots < 1 {
            anyhow::bail!("Must keep at least 1 snapshot");
        }

        // Check learning window
        if self.learning.command_history_days < 1 {
            anyhow::bail!("Command history window must be at least 1 day");
        }

        Ok(())
    }
}

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Automatically check system on daemon startup
    pub auto_check_on_startup: bool,

    /// Interval for periodic telemetry refresh (seconds)
    pub refresh_interval_seconds: u64,

    /// Enable colored output
    pub colored_output: bool,

    /// Verbosity level (0=errors, 1=warnings, 2=info, 3=debug)
    pub verbosity: u8,

    /// Enable emoji in output
    pub enable_emoji: bool,

    /// Audit log path
    pub audit_log_path: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            auto_check_on_startup: true,
            refresh_interval_seconds: 3600, // 1 hour
            colored_output: true,
            verbosity: 2, // info
            enable_emoji: true,
            audit_log_path: "/var/log/anna/audit.jsonl".to_string(),
        }
    }
}

/// Autonomy and auto-apply configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyConfig {
    /// Autonomy tier (0-3)
    pub tier: AutonomyTier,

    /// Maximum risk level to auto-apply
    pub max_auto_risk: RiskLevel,

    /// Minimum priority to auto-apply
    pub min_auto_priority: Priority,

    /// Categories allowed for auto-apply
    pub auto_apply_categories: Vec<String>,

    /// Categories explicitly blocked from auto-apply
    pub blocked_categories: Vec<String>,

    /// Require confirmation for High risk actions
    pub confirm_high_risk: bool,

    /// Create snapshot before auto-applying
    pub snapshot_before_apply: bool,
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            tier: AutonomyTier::AdviseOnly,
            max_auto_risk: RiskLevel::Low,
            min_auto_priority: Priority::Recommended,
            auto_apply_categories: vec!["security".to_string(), "maintenance".to_string()],
            blocked_categories: vec!["bootloader".to_string(), "kernel".to_string()],
            confirm_high_risk: true,
            snapshot_before_apply: true,
        }
    }
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable desktop notifications (via notify-send)
    pub desktop_notifications: bool,

    /// Enable terminal notifications (via wall)
    pub terminal_notifications: bool,

    /// Notify on new critical recommendations
    pub notify_on_critical: bool,

    /// Notify on auto-applied actions
    pub notify_on_auto_apply: bool,

    /// Notify on failed actions
    pub notify_on_failure: bool,

    /// Minimum priority for notifications
    pub min_notify_priority: Priority,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            desktop_notifications: true,
            terminal_notifications: false,
            notify_on_critical: true,
            notify_on_auto_apply: true,
            notify_on_failure: true,
            min_notify_priority: Priority::Recommended,
        }
    }
}

/// Snapshot and rollback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Enable snapshot support
    pub enabled: bool,

    /// Snapshot method: "btrfs", "timeshift", "rsync", "none"
    pub method: String,

    /// Snapshot location
    pub snapshot_path: String,

    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,

    /// Automatically create snapshot before risky operations
    pub auto_snapshot_on_risk: bool,

    /// Risk levels that trigger auto-snapshot
    pub snapshot_risk_levels: Vec<RiskLevel>,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: false, // User must explicitly enable
            method: "btrfs".to_string(),
            snapshot_path: "/.snapshots".to_string(),
            max_snapshots: 10,
            auto_snapshot_on_risk: true,
            snapshot_risk_levels: vec![RiskLevel::Medium, RiskLevel::High],
        }
    }
}

/// Learning and behavior tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Enable learning from user behavior
    pub enabled: bool,

    /// Track dismissed recommendations
    pub track_dismissed: bool,

    /// Track applied recommendations
    pub track_applied: bool,

    /// Days of command history to analyze
    pub command_history_days: u32,

    /// Minimum command usage to trigger recommendations
    pub min_command_usage: usize,

    /// Learning data storage path
    pub learning_data_path: String,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_dismissed: true,
            track_applied: true,
            command_history_days: 90,
            min_command_usage: 10,
            learning_data_path: "/var/lib/anna/learning.json".to_string(),
        }
    }
}

/// Category-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryConfig {
    /// Enabled categories (empty = all enabled)
    pub enabled: Vec<String>,

    /// Disabled categories
    pub disabled: Vec<String>,

    /// Category-specific priority overrides
    pub priority_overrides: std::collections::HashMap<String, Priority>,
}

/// User profile for multi-user systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Username
    pub username: String,

    /// User's preferred shell
    pub shell: Option<String>,

    /// User's desktop environment
    pub desktop_environment: Option<String>,

    /// User's display server
    pub display_server: Option<String>,

    /// Categories this user cares about
    pub preferred_categories: Vec<String>,

    /// Custom autonomy tier for this user
    pub autonomy_tier: Option<AutonomyTier>,
}

/// LLM user configuration (v6.54.1)
/// This is the user's preference in ~/.config/anna/config.toml
/// Separate from the internal LlmConfig in llm.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUserConfig {
    /// Preferred LLM model (e.g., "qwen2.5:14b", "llama3.1:8b")
    /// If not set or model is missing, falls back to default
    pub model: Option<String>,
}

impl Default for LlmUserConfig {
    fn default() -> Self {
        Self { model: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Config with model specified (v6.54.1)
    #[test]
    fn test_llm_config_with_model() {
        let toml_str = r#"
[llm]
model = "qwen2.5:14b"
"#;
        let config: Config = toml::from_str(toml_str).expect("Failed to parse config");
        assert_eq!(config.llm.model, Some("qwen2.5:14b".to_string()));
    }

    /// Test 2: Config without LLM section - should use default (v6.54.1)
    #[test]
    fn test_llm_config_no_section() {
        // Empty config should deserialize with all defaults
        let toml_str = "";
        let config: Config = toml::from_str(toml_str).expect("Failed to parse config");
        assert_eq!(config.llm.model, None);
    }

    /// Test 3: Config with LLM section but no model - should use default (v6.54.1)
    #[test]
    fn test_llm_config_empty_section() {
        let toml_str = r#"
[llm]
"#;
        let config: Config = toml::from_str(toml_str).expect("Failed to parse config");
        assert_eq!(config.llm.model, None);
    }

    /// Test 4: Multiple valid model names (v6.54.1)
    #[test]
    fn test_llm_config_various_models() {
        let test_cases = vec![
            "llama3.1:8b",
            "qwen2.5:14b",
            "mixtral:8x7b",
            "codellama:13b",
        ];

        for model_name in test_cases {
            let toml_str = format!(
                r#"
[llm]
model = "{}"
"#,
                model_name
            );
            let config: Config = toml::from_str(&toml_str).expect("Failed to parse config");
            assert_eq!(config.llm.model, Some(model_name.to_string()));
        }
    }
}
