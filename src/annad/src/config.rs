use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SYSTEM_CONFIG_FILE: &str = "/etc/anna/config.toml";
const DEFAULT_CONFIG: &str = include_str!("../../../config/default.toml");

/// Configuration scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    System,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub autonomy: AutonomyConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub shell: ShellConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub socket_path: String,
    pub pid_file: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: "/run/anna/annad.sock".to_string(),
            pid_file: "/run/anna/annad.pid".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyConfig {
    #[serde(default = "default_autonomy_level")]
    pub level: String, // "off" | "low" | "safe"
}

fn default_autonomy_level() -> String {
    "off".to_string()
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            level: default_autonomy_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub directory: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            directory: "/var/log/anna".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_true")]
    pub local_store: bool, // on/off (true/false)
}

fn default_true() -> bool {
    true
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            local_store: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default)]
    pub integrations: ShellIntegrations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellIntegrations {
    #[serde(default = "default_true")]
    pub autocomplete: bool, // on/off (true/false)
}

impl Default for ShellIntegrations {
    fn default() -> Self {
        Self {
            autocomplete: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("Invalid default config")
    }
}

/// Get the user config file path
fn user_config_file() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/anna/config.toml"))
}

/// Load and merge configuration from system and user files
pub fn load_config() -> Result<Config> {
    // Start with default config
    let mut config = Config::default();

    // Load system config if it exists
    if Path::new(SYSTEM_CONFIG_FILE).exists() {
        let contents = fs::read_to_string(SYSTEM_CONFIG_FILE)
            .context("Failed to read system config")?;
        config = toml::from_str(&contents)
            .context("Failed to parse system config")?;
    } else {
        // Create default system config
        save_config(&config, Scope::System)?;
    }

    // Load and merge user config if it exists
    if let Some(user_file) = user_config_file() {
        if user_file.exists() {
            let contents = fs::read_to_string(&user_file)
                .context("Failed to read user config")?;
            let user_config: Config = toml::from_str(&contents)
                .context("Failed to parse user config")?;

            // User config takes precedence
            config = merge_configs(config, user_config);
        }
    }

    Ok(config)
}

/// Merge two configs, with overlay taking precedence
fn merge_configs(base: Config, overlay: Config) -> Config {
    // For Sprint 1, we do a simple field-level merge
    // In the future, this could be more sophisticated
    Config {
        daemon: overlay.daemon,
        autonomy: overlay.autonomy,
        logging: overlay.logging,
        telemetry: overlay.telemetry,
        shell: overlay.shell,
    }
}

/// Save configuration to the specified scope
pub fn save_config(config: &Config, scope: Scope) -> Result<()> {
    let toml_str = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;

    match scope {
        Scope::System => {
            // Use polkit module for privileged write
            crate::polkit::write_system_config(SYSTEM_CONFIG_FILE, &toml_str)?;
        }
        Scope::User => {
            if let Some(user_file) = user_config_file() {
                // Ensure parent directory exists
                if let Some(parent) = user_file.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                fs::write(&user_file, toml_str)?;
            } else {
                anyhow::bail!("Cannot determine user config path");
            }
        }
    }

    Ok(())
}

/// Get a configuration value by dot-notation key
pub fn get_value(config: &Config, key: &str) -> Option<String> {
    match key {
        "autonomy.level" => Some(config.autonomy.level.clone()),
        "telemetry.local_store" => Some(if config.telemetry.local_store { "on" } else { "off" }.to_string()),
        "shell.integrations.autocomplete" => Some(if config.shell.integrations.autocomplete { "on" } else { "off" }.to_string()),
        "daemon.socket_path" => Some(config.daemon.socket_path.clone()),
        "daemon.pid_file" => Some(config.daemon.pid_file.clone()),
        "logging.level" => Some(config.logging.level.clone()),
        "logging.directory" => Some(config.logging.directory.clone()),
        _ => None,
    }
}

/// Set a configuration value by dot-notation key
pub fn set_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "autonomy.level" => {
            if !["off", "low", "safe"].contains(&value) {
                anyhow::bail!("Invalid autonomy level: must be 'off', 'low', or 'safe'");
            }
            config.autonomy.level = value.to_string();
        }
        "telemetry.local_store" => {
            config.telemetry.local_store = parse_bool(value)?;
        }
        "shell.integrations.autocomplete" => {
            config.shell.integrations.autocomplete = parse_bool(value)?;
        }
        "logging.level" => {
            config.logging.level = value.to_string();
        }
        "daemon.socket_path" => {
            config.daemon.socket_path = value.to_string();
        }
        "daemon.pid_file" => {
            config.daemon.pid_file = value.to_string();
        }
        "logging.directory" => {
            config.logging.directory = value.to_string();
        }
        _ => anyhow::bail!("Unknown configuration key: {}", key),
    }
    Ok(())
}

/// Parse boolean from string (on/off, true/false, yes/no, 1/0)
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "on" | "true" | "yes" | "1" => Ok(true),
        "off" | "false" | "no" | "0" => Ok(false),
        _ => anyhow::bail!("Invalid boolean value: use 'on' or 'off'"),
    }
}

/// List all configuration keys with their current values
pub fn list_values(config: &Config) -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert("autonomy.level".to_string(), config.autonomy.level.clone());
    map.insert("telemetry.local_store".to_string(),
        if config.telemetry.local_store { "on" } else { "off" }.to_string());
    map.insert("shell.integrations.autocomplete".to_string(),
        if config.shell.integrations.autocomplete { "on" } else { "off" }.to_string());
    map.insert("daemon.socket_path".to_string(), config.daemon.socket_path.clone());
    map.insert("daemon.pid_file".to_string(), config.daemon.pid_file.clone());
    map.insert("logging.level".to_string(), config.logging.level.clone());
    map.insert("logging.directory".to_string(), config.logging.directory.clone());

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.autonomy.level, "off");
        assert!(config.telemetry.local_store);
        assert!(config.shell.integrations.autocomplete);
    }

    #[test]
    fn test_get_value() {
        let config = Config::default();
        assert_eq!(get_value(&config, "autonomy.level"), Some("off".to_string()));
        assert_eq!(get_value(&config, "telemetry.local_store"), Some("on".to_string()));
    }

    #[test]
    fn test_set_value() {
        let mut config = Config::default();
        set_value(&mut config, "autonomy.level", "low").unwrap();
        assert_eq!(config.autonomy.level, "low");

        set_value(&mut config, "telemetry.local_store", "off").unwrap();
        assert!(!config.telemetry.local_store);
    }

    #[test]
    fn test_invalid_autonomy_level() {
        let mut config = Config::default();
        let result = set_value(&mut config, "autonomy.level", "invalid");
        assert!(result.is_err());
    }
}
