// v0.0.667: Settings Normalization (Phase 243)
// Type definitions for settings normalization

use serde::{Deserialize, Serialize};

/// Normalization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NormalizationType {
    /// Case normalization
    #[default]
    Case,
    /// Whitespace normalization
    Whitespace,
    /// Key format normalization
    KeyFormat,
    /// Value format normalization
    ValueFormat,
    /// Full normalization
    Full,
}

impl std::fmt::Display for NormalizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Case => write!(f, "case"),
            Self::Whitespace => write!(f, "whitespace"),
            Self::KeyFormat => write!(f, "key_format"),
            Self::ValueFormat => write!(f, "value_format"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Case style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaseStyle {
    /// lowercase
    #[default]
    Lower,
    /// UPPERCASE
    Upper,
    /// camelCase
    Camel,
    /// snake_case
    Snake,
    /// kebab-case
    Kebab,
}

impl std::fmt::Display for CaseStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Camel => write!(f, "camel"),
            Self::Snake => write!(f, "snake"),
            Self::Kebab => write!(f, "kebab"),
        }
    }
}

/// Normalizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizerConfig {
    /// Normalization type
    pub normalization_type: NormalizationType,
    /// Key case style
    pub key_case: CaseStyle,
    /// Trim whitespace
    pub trim_whitespace: bool,
    /// Collapse whitespace
    pub collapse_whitespace: bool,
    /// Remove empty values
    pub remove_empty: bool,
}

impl NormalizerConfig {
    /// Create new config
    pub fn new(normalization_type: NormalizationType) -> Self {
        Self {
            normalization_type,
            key_case: CaseStyle::Lower,
            trim_whitespace: true,
            collapse_whitespace: true,
            remove_empty: false,
        }
    }

    /// Set key case
    pub fn key_case(mut self, case: CaseStyle) -> Self {
        self.key_case = case;
        self
    }

    /// Set trim whitespace
    pub fn trim_whitespace(mut self, trim: bool) -> Self {
        self.trim_whitespace = trim;
        self
    }

    /// Set remove empty
    pub fn remove_empty(mut self, remove: bool) -> Self {
        self.remove_empty = remove;
        self
    }
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self::new(NormalizationType::Full)
    }
}
