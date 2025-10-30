use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "/etc/anna/config.toml";
const DEFAULT_CONFIG: &str = include_str!("../../../config/default.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub autonomy: AutonomyConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub socket_path: String,
    pub pid_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyConfig {
    pub tier: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub directory: String,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("Invalid default config")
    }
}

pub fn load_config() -> Result<Config> {
    if Path::new(CONFIG_FILE).exists() {
        let contents = fs::read_to_string(CONFIG_FILE)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    } else {
        // Create default config
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config)?;
        fs::write(CONFIG_FILE, toml_str)?;
        Ok(config)
    }
}
