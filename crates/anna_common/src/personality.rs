//! Personality Configuration - Anna's tone and verbosity settings
//!
//! Phase 5.1: Conversational UX
//! Allows users to adjust Anna's personality through natural language

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    Low,
    Normal,
    High,
}

impl Verbosity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" | "brief" | "concise" => Some(Verbosity::Low),
            "normal" | "medium" | "default" => Some(Verbosity::Normal),
            "high" | "detailed" | "verbose" => Some(Verbosity::High),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Verbosity::Low => "low",
            Verbosity::Normal => "normal",
            Verbosity::High => "high",
        }
    }
}

/// Personality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// Humor level: 0 = serious, 1 = moderate, 2 = playful
    #[serde(default = "default_humor")]
    pub humor_level: u8,

    /// Verbosity level
    #[serde(default = "default_verbosity")]
    pub verbosity: Verbosity,
}

fn default_humor() -> u8 {
    1 // Moderate humor by default
}

fn default_verbosity() -> Verbosity {
    Verbosity::Normal
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            humor_level: default_humor(),
            verbosity: default_verbosity(),
        }
    }
}

impl PersonalityConfig {
    /// Load personality config from file
    /// Checks ~/.config/anna/personality.toml first, then /etc/anna/personality.toml
    pub fn load() -> Self {
        // Try user config first
        if let Some(path) = Self::user_config_path() {
            if let Ok(config) = Self::load_from_path(&path) {
                return config;
            }
        }

        // Try system config
        if let Ok(config) = Self::load_from_path(&PathBuf::from("/etc/anna/personality.toml")) {
            return config;
        }

        // Default if neither exists
        Self::default()
    }

    /// Load from specific path
    fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: PersonalityConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save to user config
    pub fn save(&self) -> Result<()> {
        let path = Self::user_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get user config path: ~/.config/anna/personality.toml
    fn user_config_path() -> Option<PathBuf> {
        let config_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else {
            let home = std::env::var("HOME").ok()?;
            PathBuf::from(home).join(".config")
        };

        Some(config_dir.join("anna").join("personality.toml"))
    }

    /// Adjust humor level up or down
    pub fn adjust_humor(&mut self, increase: bool) {
        if increase {
            self.humor_level = (self.humor_level + 1).min(2);
        } else {
            self.humor_level = self.humor_level.saturating_sub(1);
        }
    }

    /// Set verbosity
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    /// Check if humor is enabled
    pub fn has_humor(&self) -> bool {
        self.humor_level > 0
    }

    /// Get humor description
    pub fn humor_description(&self) -> &'static str {
        match self.humor_level {
            0 => "Serious (no jokes)",
            1 => "Moderate (subtle humor)",
            2 => "Playful (more ironic)",
            _ => "Unknown",
        }
    }

    /// Get verbosity description
    pub fn verbosity_description(&self) -> &'static str {
        match self.verbosity {
            Verbosity::Low => "Brief (concise answers)",
            Verbosity::Normal => "Normal (balanced)",
            Verbosity::High => "Detailed (thorough explanations)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_personality() {
        let config = PersonalityConfig::default();
        assert_eq!(config.humor_level, 1);
        assert_eq!(config.verbosity, Verbosity::Normal);
    }

    #[test]
    fn test_humor_adjustment() {
        let mut config = PersonalityConfig::default();

        // Increase
        config.adjust_humor(true);
        assert_eq!(config.humor_level, 2);

        // Cap at 2
        config.adjust_humor(true);
        assert_eq!(config.humor_level, 2);

        // Decrease
        config.adjust_humor(false);
        assert_eq!(config.humor_level, 1);

        config.adjust_humor(false);
        assert_eq!(config.humor_level, 0);

        // Floor at 0
        config.adjust_humor(false);
        assert_eq!(config.humor_level, 0);
    }

    #[test]
    fn test_verbosity_parsing() {
        assert_eq!(Verbosity::from_str("low"), Some(Verbosity::Low));
        assert_eq!(Verbosity::from_str("brief"), Some(Verbosity::Low));
        assert_eq!(Verbosity::from_str("normal"), Some(Verbosity::Normal));
        assert_eq!(Verbosity::from_str("high"), Some(Verbosity::High));
        assert_eq!(Verbosity::from_str("verbose"), Some(Verbosity::High));
        assert_eq!(Verbosity::from_str("invalid"), None);
    }

    #[test]
    fn test_has_humor() {
        let mut config = PersonalityConfig::default();
        config.humor_level = 0;
        assert!(!config.has_humor());

        config.humor_level = 1;
        assert!(config.has_humor());

        config.humor_level = 2;
        assert!(config.has_humor());
    }
}
