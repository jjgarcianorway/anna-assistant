//! Persistent configuration for Anna.
//!
//! Stored at ~/.anna/config.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Anna configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaConfig {
    /// Debug mode - shows full prompts
    #[serde(default = "default_true")]
    pub debug_mode: bool,

    /// Auto-install helpers when needed
    #[serde(default = "default_true")]
    pub auto_install_helpers: bool,

    /// Ask for clarification on ambiguous queries
    #[serde(default = "default_true")]
    pub ask_clarification: bool,

    /// Wiki settings
    #[serde(default)]
    pub wiki: WikiConfig,
}

fn default_true() -> bool {
    true
}

/// Wiki configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiConfig {
    /// Path to local wiki cache
    #[serde(default = "default_wiki_path")]
    pub cache_path: PathBuf,

    /// Use embeddings for semantic search
    #[serde(default = "default_true")]
    pub use_embeddings: bool,

    /// Last sync time
    pub last_sync: Option<String>,
}

fn default_wiki_path() -> PathBuf {
    anna_data_dir().join("wiki")
}

impl Default for AnnaConfig {
    fn default() -> Self {
        Self {
            debug_mode: true,
            auto_install_helpers: true,
            ask_clarification: true,
            wiki: WikiConfig::default(),
        }
    }
}

impl AnnaConfig {
    /// Load config from disk, or create default
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AnnaConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AnnaConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Update a setting by name (for natural language control)
    pub fn set(&mut self, key: &str, value: bool) -> Result<()> {
        match key {
            "debug_mode" | "debug" => self.debug_mode = value,
            "auto_install_helpers" | "auto_install" => self.auto_install_helpers = value,
            "ask_clarification" | "clarification" => self.ask_clarification = value,
            "use_embeddings" | "embeddings" => self.wiki.use_embeddings = value,
            _ => anyhow::bail!("Unknown setting: {}", key),
        }
        self.save()?;
        Ok(())
    }
}

/// Get Anna data directory (~/.anna)
pub fn anna_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
}

/// Get config file path
pub fn config_path() -> PathBuf {
    anna_data_dir().join("config.toml")
}
