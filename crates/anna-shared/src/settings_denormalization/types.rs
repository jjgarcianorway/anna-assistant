// v0.0.668: Settings Denormalization Types
// Core types for denormalization

use serde::{Deserialize, Serialize};

/// Denormalization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DenormalizationType {
    /// Expand to target format
    #[default]
    Expand,
    /// Unflatten nested structure
    Unflatten,
    /// Add prefixes
    Prefix,
    /// Add suffixes
    Suffix,
    /// Full denormalization
    Full,
}

impl std::fmt::Display for DenormalizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expand => write!(f, "expand"),
            Self::Unflatten => write!(f, "unflatten"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Target format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetFormat {
    /// JSON format
    #[default]
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// INI format
    Ini,
    /// Env format
    Env,
}

impl std::fmt::Display for TargetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denorm_type_display() {
        assert_eq!(format!("{}", DenormalizationType::Expand), "expand");
        assert_eq!(format!("{}", DenormalizationType::Unflatten), "unflatten");
    }

    #[test]
    fn test_target_format_display() {
        assert_eq!(format!("{}", TargetFormat::Json), "json");
        assert_eq!(format!("{}", TargetFormat::Yaml), "yaml");
    }
}
