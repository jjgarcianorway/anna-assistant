// v0.0.555: Settings Persistence - Format types (Phase 131)
// Defines serialization formats for settings

use serde::{Deserialize, Serialize};

/// Settings file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SettingsFormat {
    /// JSON format (default)
    #[default]
    Json,
    /// TOML format
    Toml,
}

impl std::fmt::Display for SettingsFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Toml => write!(f, "TOML"),
        }
    }
}
