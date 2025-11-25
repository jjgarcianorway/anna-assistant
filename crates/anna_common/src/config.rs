//! Configuration management for Anna Assistant
//!
//! Handles loading, saving, and validating user configuration from ~/.config/anna/config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::types::{AutonomyTier, Priority, RiskLevel};

/// Main configuration structure
/// v6.54.4: Added #[serde(flatten)] to allow unknown sections in config
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

    /// Output configuration (v6.54.4)
    #[serde(default)]
    pub output: OutputConfig,

    /// Catch-all for unknown sections to prevent parse failures
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, toml::Value>,
}

impl Config {
    /// Get the default config file path: ~/.config/anna/config.toml
    pub fn default_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let config_dir = Path::new(&home).join(".config").join("anna");
        Ok(config_dir.join("config.toml"))
    }

    /// Get system-wide config path: /etc/anna/config.toml
    pub fn system_path() -> PathBuf {
        PathBuf::from("/etc/anna/config.toml")
    }

    /// Get config path for the real user (when running as root via systemd)
    /// v6.54.5: Always scan /home/* for user configs when running as a system service
    pub fn real_user_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok();
        let uid = unsafe { libc::getuid() };
        info!("real_user_path: HOME={:?}, uid={}", home, uid);

        // If running as root (uid 0), look for user configs
        if uid == 0 {
            info!("Running as root (uid=0), scanning /home for user configs...");

            // Try SUDO_USER environment variable first (if running via sudo)
            if let Ok(sudo_user) = std::env::var("SUDO_USER") {
                let user_home = format!("/home/{}", sudo_user);
                let path = Path::new(&user_home)
                    .join(".config")
                    .join("anna")
                    .join("config.toml");
                info!("Checking SUDO_USER path: {:?} exists={}", path, path.exists());
                if path.exists() {
                    info!("Found config via SUDO_USER: {:?}", path);
                    return Some(path);
                }
            }

            // Scan /home/*/.config/anna/config.toml
            match std::fs::read_dir("/home") {
                Ok(entries) => {
                    let mut found_any = false;
                    for entry_result in entries {
                        match entry_result {
                            Ok(entry) => {
                                found_any = true;
                                let path = entry
                                    .path()
                                    .join(".config")
                                    .join("anna")
                                    .join("config.toml");
                                info!("Checking: {:?} exists={}", path, path.exists());
                                if path.exists() {
                                    info!("Found user config at: {:?}", path);
                                    return Some(path);
                                }
                            }
                            Err(e) => {
                                info!("Error reading /home entry: {}", e);
                            }
                        }
                    }
                    if !found_any {
                        info!("No entries found in /home");
                    } else {
                        info!("No user configs found in /home");
                    }
                }
                Err(e) => {
                    info!("Failed to read /home directory: {}", e);
                }
            }
        } else {
            info!("Not running as root (uid={}), using standard config path", uid);
        }
        None
    }

    /// Load configuration from file, or create default if not exists
    /// v6.54.2: When running as root (daemon), also checks real user's config and /etc/anna/
    pub fn load() -> Result<Self> {
        Self::load_with_path().map(|(config, _)| config)
    }

    /// Load configuration and return which file it was loaded from
    /// v6.54.3: Returns (Config, Option<PathBuf>) for debugging
    pub fn load_with_path() -> Result<(Self, Option<PathBuf>)> {
        let home = std::env::var("HOME").unwrap_or_default();
        debug!("Config::load() - HOME={}", home);

        // Priority 1: Real user's config (when running as root daemon)
        if let Some(user_path) = Self::real_user_path() {
            debug!("Checking real user config: {:?}", user_path);
            if let Ok(contents) = fs::read_to_string(&user_path) {
                match toml::from_str(&contents) {
                    Ok(config) => {
                        info!("Loaded config from real user path: {:?}", user_path);
                        return Ok((config, Some(user_path)));
                    }
                    Err(e) => {
                        // v6.54.3: Log parse errors at INFO level so they're visible
                        info!("Failed to parse {:?}: {}", user_path, e);
                    }
                }
            }
        }

        // Priority 2: System-wide config
        let system_path = Self::system_path();
        debug!("Checking system config: {:?}", system_path);
        if system_path.exists() {
            if let Ok(contents) = fs::read_to_string(&system_path) {
                match toml::from_str(&contents) {
                    Ok(config) => {
                        info!("Loaded config from system path: {:?}", system_path);
                        return Ok((config, Some(system_path)));
                    }
                    Err(e) => {
                        info!("Failed to parse {:?}: {}", system_path, e);
                    }
                }
            }
        }

        // Priority 3: User's own config (~/.config/anna/config.toml)
        let path = Self::default_path()?;
        debug!("Checking default config: {:?}", path);
        if path.exists() {
            let contents = fs::read_to_string(&path).context("Failed to read config file")?;
            let config: Config =
                toml::from_str(&contents).context("Failed to parse config file")?;
            info!("Loaded config from default path: {:?}", path);
            Ok((config, Some(path)))
        } else {
            // No config found - return defaults
            // v6.54.2: Don't try to save when running as root (daemon), just return defaults
            debug!("No config found, using defaults (HOME={})", home);
            Ok((Config::default(), None))
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

/// Output configuration (v6.54.4)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    /// Emoji display: "enabled", "disabled", "auto"
    #[serde(default)]
    pub emojis: Option<String>,

    /// Color output: "always", "never", "auto"
    #[serde(default)]
    pub color: Option<String>,
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
