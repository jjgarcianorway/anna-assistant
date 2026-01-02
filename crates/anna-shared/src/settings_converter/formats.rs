// v0.0.650: Settings Converter Formats (Phase 226)
// Format enums for settings conversion

use serde::{Deserialize, Serialize};

/// Source format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SourceFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// INI format
    Ini,
    /// Env format
    Env,
}

impl std::fmt::Display for SourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

/// Target format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TargetFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// INI format
    Ini,
    /// Env format
    Env,
}

impl std::fmt::Display for TargetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}
