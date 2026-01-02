// v0.0.673: Settings Selector Types (Phase 249)
// Enums and type definitions for settings selector

use serde::{Deserialize, Serialize};

/// Selector type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SelectorType {
    /// Select by key pattern
    #[default]
    Pattern,
    /// Select by value
    ByValue,
    /// Select by index/position
    ByIndex,
    /// Select first N
    First,
    /// Select last N
    Last,
}

impl std::fmt::Display for SelectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::ByValue => write!(f, "by_value"),
            Self::ByIndex => write!(f, "by_index"),
            Self::First => write!(f, "first"),
            Self::Last => write!(f, "last"),
        }
    }
}

/// Match mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MatchMode {
    /// Exact match
    #[default]
    Exact,
    /// Prefix match
    Prefix,
    /// Suffix match
    Suffix,
    /// Contains
    Contains,
    /// Regex match
    Regex,
}

impl std::fmt::Display for MatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Contains => write!(f, "contains"),
            Self::Regex => write!(f, "regex"),
        }
    }
}
