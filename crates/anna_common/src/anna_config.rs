//! Anna Configuration System (6.18.0)
//!
//! User configuration for output preferences and behavior.
//! Config file: ~/.config/anna/config.toml or /etc/anna/config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Emoji display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmojiMode {
    /// Auto-detect based on terminal capabilities
    Auto,
    /// Always show emojis
    Enabled,
    /// Never show emojis (ASCII fallback)
    Disabled,
}

impl Default for EmojiMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Color display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    /// Auto-detect based on terminal capabilities
    Auto,
    /// Force basic ANSI colors (8/16 colors)
    Basic,
    /// No colors (plain text)
    None,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Emoji display mode
    #[serde(default)]
    pub emojis: EmojiMode,

    /// Color display mode
    #[serde(default)]
    pub color: ColorMode,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            emojis: EmojiMode::Auto,
            color: ColorMode::Auto,
        }
    }
}

/// Developer/debug configuration (v10.2.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    /// Show timing diagnostics for each query
    /// Can also be enabled via ANNA_DEV_DEBUG=1
    #[serde(default)]
    pub show_timing: bool,

    /// Log chain-of-thought reasoning to file
    /// Logs to ~/.local/share/anna/reasoning.log
    #[serde(default)]
    pub log_reasoning: bool,

    /// Show LLM orchestrator steps in output
    #[serde(default)]
    pub show_steps: bool,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            show_timing: false,
            log_reasoning: false,
            show_steps: false,
        }
    }
}

impl DevConfig {
    /// Check if dev debug is enabled (config or env)
    pub fn is_debug_enabled(&self) -> bool {
        self.show_timing
            || std::env::var("ANNA_DEV_DEBUG").map(|v| v == "1").unwrap_or(false)
    }

    /// Check if reasoning logging is enabled
    pub fn should_log_reasoning(&self) -> bool {
        self.log_reasoning
            || std::env::var("ANNA_LOG_REASONING").map(|v| v == "1").unwrap_or(false)
    }
}

/// Main Anna configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaConfig {
    /// Output preferences
    #[serde(default)]
    pub output: OutputConfig,

    /// Developer/debug settings (v10.2.1)
    #[serde(default)]
    pub dev: DevConfig,
}

impl Default for AnnaConfig {
    fn default() -> Self {
        Self {
            output: OutputConfig::default(),
            dev: DevConfig::default(),
        }
    }
}

impl AnnaConfig {
    /// Get default user config path: ~/.config/anna/config.toml
    pub fn user_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("XDG_CONFIG_HOME"))
            .context("Cannot determine home directory")?;

        let config_dir = if home.contains("/.config") {
            PathBuf::from(home)
        } else {
            Path::new(&home).join(".config")
        };

        Ok(config_dir.join("anna").join("config.toml"))
    }

    /// Get system config path: /etc/anna/config.toml
    pub fn system_config_path() -> PathBuf {
        PathBuf::from("/etc/anna/config.toml")
    }

    /// Load configuration from file
    ///
    /// Priority:
    /// 1. User config (~/.config/anna/config.toml)
    /// 2. System config (/etc/anna/config.toml)
    /// 3. Defaults
    pub fn load() -> Result<Self> {
        // Try user config first
        if let Ok(user_path) = Self::user_config_path() {
            if user_path.exists() {
                let contents = fs::read_to_string(&user_path)
                    .with_context(|| format!("Failed to read {}", user_path.display()))?;
                let config: AnnaConfig = toml::from_str(&contents)
                    .with_context(|| format!("Failed to parse {}", user_path.display()))?;
                return Ok(config);
            }
        }

        // Try system config
        let system_path = Self::system_config_path();
        if system_path.exists() {
            let contents = fs::read_to_string(&system_path)
                .with_context(|| format!("Failed to read {}", system_path.display()))?;
            let config: AnnaConfig = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse {}", system_path.display()))?;
            return Ok(config);
        }

        // Return defaults
        Ok(Self::default())
    }

    /// Save configuration to user config file
    pub fn save(&self) -> Result<()> {
        let path = Self::user_config_path()?;

        // Create parent directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        // Serialize to TOML
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        // Write to file
        fs::write(&path, toml_string)
            .with_context(|| format!("Failed to write {}", path.display()))?;

        Ok(())
    }

    /// Set output emoji mode
    pub fn set_emoji_mode(&mut self, mode: &str) -> Result<()> {
        self.output.emojis = match mode.to_lowercase().as_str() {
            "auto" => EmojiMode::Auto,
            "on" | "enabled" | "yes" | "true" => EmojiMode::Enabled,
            "off" | "disabled" | "no" | "false" => EmojiMode::Disabled,
            _ => anyhow::bail!("Invalid emoji mode: '{}'. Valid values: auto, on, off", mode),
        };
        Ok(())
    }

    /// Set output color mode
    pub fn set_color_mode(&mut self, mode: &str) -> Result<()> {
        self.output.color = match mode.to_lowercase().as_str() {
            "auto" => ColorMode::Auto,
            "basic" => ColorMode::Basic,
            "none" | "off" | "disabled" => ColorMode::None,
            _ => anyhow::bail!("Invalid color mode: '{}'. Valid values: auto, basic, none", mode),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnnaConfig::default();
        assert_eq!(config.output.emojis, EmojiMode::Auto);
        assert_eq!(config.output.color, ColorMode::Auto);
    }

    #[test]
    fn test_emoji_mode_parsing() {
        let mut config = AnnaConfig::default();

        config.set_emoji_mode("auto").unwrap();
        assert_eq!(config.output.emojis, EmojiMode::Auto);

        config.set_emoji_mode("on").unwrap();
        assert_eq!(config.output.emojis, EmojiMode::Enabled);

        config.set_emoji_mode("off").unwrap();
        assert_eq!(config.output.emojis, EmojiMode::Disabled);

        assert!(config.set_emoji_mode("invalid").is_err());
    }

    #[test]
    fn test_color_mode_parsing() {
        let mut config = AnnaConfig::default();

        config.set_color_mode("auto").unwrap();
        assert_eq!(config.output.color, ColorMode::Auto);

        config.set_color_mode("basic").unwrap();
        assert_eq!(config.output.color, ColorMode::Basic);

        config.set_color_mode("none").unwrap();
        assert_eq!(config.output.color, ColorMode::None);

        assert!(config.set_color_mode("invalid").is_err());
    }

    #[test]
    fn test_toml_serialization() {
        let config = AnnaConfig::default();
        let toml = toml::to_string(&config).unwrap();

        assert!(toml.contains("[output]"));
        assert!(toml.contains("emojis"));
        assert!(toml.contains("color"));
    }

    #[test]
    fn test_toml_round_trip() {
        let original = AnnaConfig {
            output: OutputConfig {
                emojis: EmojiMode::Disabled,
                color: ColorMode::Basic,
            },
            dev: DevConfig::default(),
        };

        let toml = toml::to_string(&original).unwrap();
        let parsed: AnnaConfig = toml::from_str(&toml).unwrap();

        assert_eq!(parsed.output.emojis, EmojiMode::Disabled);
        assert_eq!(parsed.output.color, ColorMode::Basic);
    }

    #[test]
    fn test_dev_config_debug_enabled() {
        let mut dev = DevConfig::default();
        assert!(!dev.is_debug_enabled());

        dev.show_timing = true;
        assert!(dev.is_debug_enabled());
    }
}
