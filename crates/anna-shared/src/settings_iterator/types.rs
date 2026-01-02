// v0.0.681: Settings Iterator Types (Phase 257)
// Core enums for settings iteration

use serde::{Deserialize, Serialize};

/// Iteration order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IterationOrder {
    /// Natural order (HashMap default)
    #[default]
    Natural,
    /// Alphabetical by key
    Alphabetical,
    /// Reverse alphabetical
    ReverseAlphabetical,
    /// By value length
    ByValueLength,
}

impl std::fmt::Display for IterationOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Natural => write!(f, "natural"),
            Self::Alphabetical => write!(f, "alphabetical"),
            Self::ReverseAlphabetical => write!(f, "reverse_alphabetical"),
            Self::ByValueLength => write!(f, "by_value_length"),
        }
    }
}

/// Iteration filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IterationFilter {
    /// No filter
    #[default]
    None,
    /// Only non-empty values
    NonEmpty,
    /// Only numeric values
    Numeric,
    /// Only boolean values
    Boolean,
}

impl std::fmt::Display for IterationFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::NonEmpty => write!(f, "non_empty"),
            Self::Numeric => write!(f, "numeric"),
            Self::Boolean => write!(f, "boolean"),
        }
    }
}
