// v0.0.649: Settings Decoder Types (Phase 225)
// Enums and basic types for settings decoder

use serde::{Deserialize, Serialize};

/// Decoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecodingFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Binary format
    Binary,
    /// Base64 format
    Base64,
}

impl std::fmt::Display for DecodingFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Binary => write!(f, "binary"),
            Self::Base64 => write!(f, "base64"),
        }
    }
}

/// Decoding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DecodingMode {
    /// Strict mode
    #[default]
    Strict,
    /// Lenient mode
    Lenient,
    /// Permissive mode
    Permissive,
    /// Recovery mode
    Recovery,
}

impl std::fmt::Display for DecodingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Lenient => write!(f, "lenient"),
            Self::Permissive => write!(f, "permissive"),
            Self::Recovery => write!(f, "recovery"),
        }
    }
}
