// Anna v0.12.7 - Configuration Reload Module
// Hot-reload configuration without daemon restart

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Anna daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnaConfig {
    /// Autonomy level settings
    #[serde(default)]
    pub autonomy: AutonomyConfig,

    /// UI preferences
    #[serde(default)]
    pub ui: UiConfig,

    /// Telemetry collection settings
    #[serde(default)]
    pub telemetry: TelemetryConfig,

    /// Persona settings
    #[serde(default)]
    pub persona: PersonaConfig,

    /// Daemon operational settings
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutonomyConfig {
    #[serde(default = "default_autonomy_level")]
    pub level: String, // "low" or "high"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    #[serde(default = "default_false")]
    pub emojis: bool,

    #[serde(default = "default_true")]
    pub color: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,

    #[serde(default = "default_collection_interval")]
    pub collection_interval_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonaConfig {
    #[serde(default = "default_active_persona")]
    pub active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonConfig {
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,

    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,

    #[serde(default = "default_poll_jitter")]
    pub poll_jitter_secs: u64,
}

// Default value functions
fn default_autonomy_level() -> String {
    "low".to_string()
}
fn default_false() -> bool {
    false
}
fn default_true() -> bool {
    true
}
fn default_collection_interval() -> u64 {
    60
}
fn default_active_persona() -> String {
    "dev".to_string()
}
fn default_db_path() -> PathBuf {
    PathBuf::from("/var/lib/anna/telemetry.db")
}
fn default_socket_path() -> PathBuf {
    PathBuf::from("/run/anna/annad.sock")
}
fn default_poll_interval() -> u64 {
    30
}
fn default_poll_jitter() -> u64 {
    5
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            level: default_autonomy_level(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            emojis: default_false(),
            color: default_true(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            collection_interval_sec: default_collection_interval(),
        }
    }
}

impl Default for PersonaConfig {
    fn default() -> Self {
        Self {
            active: default_active_persona(),
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            socket_path: default_socket_path(),
            poll_interval_secs: default_poll_interval(),
            poll_jitter_secs: default_poll_jitter(),
        }
    }
}

impl Default for AnnaConfig {
    fn default() -> Self {
        Self {
            autonomy: AutonomyConfig::default(),
            ui: UiConfig::default(),
            telemetry: TelemetryConfig::default(),
            persona: PersonaConfig::default(),
            daemon: DaemonConfig::default(),
        }
    }
}

/// Thread-safe configuration manager
pub struct ConfigManager {
    config: Arc<RwLock<AnnaConfig>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new config manager and load initial configuration
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        })
    }

    /// Load configuration from file
    fn load_config(path: &Path) -> Result<AnnaConfig> {
        if !path.exists() {
            warn!("Config file not found at {:?}, using defaults", path);
            return Ok(AnnaConfig::default());
        }

        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: AnnaConfig = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        info!("Loaded configuration from {:?}", path);

        Ok(config)
    }

    /// Validate configuration
    fn validate_config(config: &AnnaConfig) -> Result<()> {
        // Validate autonomy level
        if config.autonomy.level != "low" && config.autonomy.level != "high" {
            anyhow::bail!(
                "Invalid autonomy level: {} (must be 'low' or 'high')",
                config.autonomy.level
            );
        }

        // Validate collection interval
        if config.telemetry.collection_interval_sec == 0 {
            anyhow::bail!("Telemetry collection interval must be > 0");
        }

        if config.telemetry.collection_interval_sec > 3600 {
            warn!(
                "Telemetry collection interval very high: {}s",
                config.telemetry.collection_interval_sec
            );
        }

        // Validate poll intervals
        if config.daemon.poll_interval_secs == 0 {
            anyhow::bail!("Poll interval must be > 0");
        }

        if config.daemon.poll_jitter_secs > config.daemon.poll_interval_secs {
            anyhow::bail!("Poll jitter cannot exceed poll interval");
        }

        // Validate paths exist (parent directories)
        if let Some(parent) = config.daemon.db_path.parent() {
            if !parent.exists() {
                warn!(
                    "Database parent directory does not exist: {:?}",
                    parent
                );
            }
        }

        Ok(())
    }

    /// Reload configuration from disk
    pub async fn reload(&self) -> Result<()> {
        info!("Reloading configuration from {:?}", self.config_path);

        // Load new config
        let new_config = Self::load_config(&self.config_path)?;

        // Validate new config
        Self::validate_config(&new_config)
            .context("Configuration validation failed - keeping old config")?;

        // Get old config for comparison
        let old_config = self.config.read().await.clone();

        // Apply new config
        *self.config.write().await = new_config.clone();

        info!("Configuration reloaded successfully");

        // Log changes
        self.log_config_changes(&old_config, &new_config);

        Ok(())
    }

    /// Log configuration changes
    fn log_config_changes(&self, old: &AnnaConfig, new: &AnnaConfig) {
        if old.autonomy != new.autonomy {
            info!(
                "Autonomy level changed: {} -> {}",
                old.autonomy.level, new.autonomy.level
            );
        }

        if old.telemetry != new.telemetry {
            info!(
                "Telemetry settings changed: enabled={} interval={}s",
                new.telemetry.enabled, new.telemetry.collection_interval_sec
            );
        }

        if old.daemon.poll_interval_secs != new.daemon.poll_interval_secs {
            info!(
                "Poll interval changed: {}s -> {}s",
                old.daemon.poll_interval_secs, new.daemon.poll_interval_secs
            );
        }

        if old.persona != new.persona {
            info!(
                "Active persona changed: {} -> {}",
                old.persona.active, new.persona.active
            );
        }
    }

    /// Get a read-only reference to the current configuration
    pub async fn get(&self) -> AnnaConfig {
        self.config.read().await.clone()
    }

    /// Get Arc to config for sharing
    pub fn config_arc(&self) -> Arc<RwLock<AnnaConfig>> {
        Arc::clone(&self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AnnaConfig::default();
        assert_eq!(config.autonomy.level, "low");
        assert_eq!(config.telemetry.collection_interval_sec, 60);
        assert_eq!(config.daemon.poll_interval_secs, 30);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AnnaConfig::default();

        // Valid config
        assert!(ConfigManager::validate_config(&config).is_ok());

        // Invalid autonomy level
        config.autonomy.level = "invalid".to_string();
        assert!(ConfigManager::validate_config(&config).is_err());

        config.autonomy.level = "low".to_string();

        // Invalid collection interval
        config.telemetry.collection_interval_sec = 0;
        assert!(ConfigManager::validate_config(&config).is_err());
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = AnnaConfig::default();

        // Serialize to TOML
        let toml = toml::to_string(&config).unwrap();

        // Deserialize back
        let parsed: AnnaConfig = toml::from_str(&toml).unwrap();

        assert_eq!(config, parsed);
    }

    #[tokio::test]
    async fn test_config_manager() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let config_path = temp_file.path();

        // Write test config
        let test_config = AnnaConfig {
            autonomy: AutonomyConfig {
                level: "high".to_string(),
            },
            ..Default::default()
        };

        std::fs::write(config_path, toml::to_string(&test_config)?)?;

        // Load config
        let manager = ConfigManager::new(config_path)?;
        let loaded = manager.get().await;

        assert_eq!(loaded.autonomy.level, "high");

        Ok(())
    }
}
