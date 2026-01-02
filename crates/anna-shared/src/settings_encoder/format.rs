// v0.0.648: Settings Encoder (Phase 224)
// Encoding format types and options

use serde::{Deserialize, Serialize};

/// Encoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EncodingFormat {
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

impl std::fmt::Display for EncodingFormat {
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

/// Encoding options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncodingOptions {
    /// Compact encoding
    #[default]
    Compact,
    /// Pretty encoding
    Pretty,
    /// Minified encoding
    Minified,
    /// Verbose encoding
    Verbose,
}

impl std::fmt::Display for EncodingOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => write!(f, "compact"),
            Self::Pretty => write!(f, "pretty"),
            Self::Minified => write!(f, "minified"),
            Self::Verbose => write!(f, "verbose"),
        }
    }
}
