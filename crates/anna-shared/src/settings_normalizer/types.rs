// v0.0.645: Settings Normalizer Types (Phase 221)
// Type definitions for settings normalization

use serde::{Deserialize, Serialize};

/// Normalization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NormalizationType {
    /// String normalization
    #[default]
    String,
    /// Path normalization
    Path,
    /// URL normalization
    Url,
    /// Number normalization
    Number,
    /// Boolean normalization
    Boolean,
}

impl std::fmt::Display for NormalizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Path => write!(f, "path"),
            Self::Url => write!(f, "url"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
        }
    }
}

/// Normalization rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NormalizationRule {
    /// No transformation
    #[default]
    None,
    /// Lowercase
    Lowercase,
    /// Uppercase
    Uppercase,
    /// Trim
    Trim,
    /// Canonical form
    Canonical,
}

impl std::fmt::Display for NormalizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Lowercase => write!(f, "lowercase"),
            Self::Uppercase => write!(f, "uppercase"),
            Self::Trim => write!(f, "trim"),
            Self::Canonical => write!(f, "canonical"),
        }
    }
}
