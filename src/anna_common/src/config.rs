//! Configuration management for Anna's messaging and behavior

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Anna's user configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaConfig {
    /// Enable colored output
    #[serde(default = "default_true")]
    pub colors: bool,

    /// Enable emoji/Unicode characters
    #[serde(default = "default_true")]
    pub emojis: bool,

    /// Show timestamps and extra context
    #[serde(default = "default_true")]
    pub verbose: bool,

    /// Ask before using sudo/privilege escalation
    #[serde(default = "default_true")]
    pub confirm_privilege: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AnnaConfig {
    fn default() -> Self {
        default_config()
    }
}

/// Get default configuration
pub fn default_config() -> AnnaConfig {
    AnnaConfig {
        colors: true,
        emojis: true,
        verbose: true,
        confirm_privilege: true,
    }
}

/// Get user config file path (~/.config/anna/config.yml)
pub fn config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;

    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("anna");
    path.push("config.yml");

    Ok(path)
}

/// Load configuration from file, or return default if not found
pub fn load_config() -> AnnaConfig {
    match config_path() {
        Ok(path) => {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_yaml::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            eprintln!("Warning: Failed to parse config: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Warning: Failed to read config: {}", e);
                    }
                }
            }
        }
        Err(_) => {}
    }

    default_config()
}

/// Save configuration to file
pub fn save_config(config: &AnnaConfig) -> Result<()> {
    let path = config_path()?;

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let yaml = serde_yaml::to_string(config).context("Failed to serialize config")?;

    fs::write(&path, yaml).context("Failed to write config file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert!(config.colors);
        assert!(config.emojis);
        assert!(config.verbose);
        assert!(config.confirm_privilege);
    }

    #[test]
    fn test_serialization() {
        let config = default_config();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: AnnaConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(config.colors, deserialized.colors);
        assert_eq!(config.emojis, deserialized.emojis);
    }
}
