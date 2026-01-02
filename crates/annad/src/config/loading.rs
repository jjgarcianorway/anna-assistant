//! Configuration loading and saving logic for annad.
//!
//! Handles reading config from disk and providing defaults.

use super::types::{Config, CONFIG_PATH, DEFAULT_CONFIG_PATH};
use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

impl Config {
    /// Get debug mode setting
    pub fn debug_mode(&self) -> bool {
        self.daemon.debug_mode
    }

    /// Load config from file, or return defaults
    pub fn load() -> Self {
        Self::load_from_path(CONFIG_PATH)
            .or_else(|_| Self::load_from_path(DEFAULT_CONFIG_PATH))
            .unwrap_or_else(|e| {
                warn!("Config not found, using defaults: {}", e);
                Config::default()
            })
    }

    /// Load config from specific path
    fn load_from_path(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        info!("Loaded config from {}", path);
        Ok(config)
    }

    /// Save default config to path (for init)
    #[allow(dead_code)]
    pub fn save_default(path: &str) -> Result<()> {
        let config = Config::default();
        let content = toml::to_string_pretty(&config)?;
        // v0.0.291: Safe path handling - avoid panic on malformed paths
        let parent = Path::new(path)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid config path: {}", path))?;
        fs::create_dir_all(parent)?;
        fs::write(path, content)?;
        info!("Saved default config to {}", path);
        Ok(())
    }

    /// Get list of unique models needed (for pulling)
    /// v0.0.277: Now includes junior and senior models
    pub fn required_models(&self) -> Vec<String> {
        let mut models = vec![
            self.llm.translator_model.clone(),
            self.llm.junior_model.clone(),
            self.llm.senior_model.clone(),
        ];
        // Add supervisor only if different from others
        if !models.contains(&self.llm.supervisor_model) {
            models.push(self.llm.supervisor_model.clone());
        }
        models.sort();
        models.dedup();
        models
    }
}
