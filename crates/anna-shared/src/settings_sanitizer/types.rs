// v0.0.643: Settings Sanitizer Types (Phase 219)
// Type definitions for sanitization

use serde::{Deserialize, Serialize};

/// Sanitization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SanitizationType {
    /// Trim whitespace
    #[default]
    Trim,
    /// Normalize case
    NormalizeCase,
    /// Remove special chars
    RemoveSpecial,
    /// Escape values
    Escape,
    /// Full sanitization
    Full,
}

impl std::fmt::Display for SanitizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trim => write!(f, "trim"),
            Self::NormalizeCase => write!(f, "normalize_case"),
            Self::RemoveSpecial => write!(f, "remove_special"),
            Self::Escape => write!(f, "escape"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Case normalization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaseNormalization {
    /// No change
    #[default]
    None,
    /// Lowercase
    Lower,
    /// Uppercase
    Upper,
    /// Title case
    Title,
}

impl std::fmt::Display for CaseNormalization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Title => write!(f, "title"),
        }
    }
}
