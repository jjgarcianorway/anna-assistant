// v0.0.661: Settings Differ Types (Phase 237)
// Core types for settings differ

use serde::{Deserialize, Serialize};

/// Diff type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiffType {
    /// Added key
    Added,
    /// Removed key
    Removed,
    /// Modified value
    #[default]
    Modified,
    /// Unchanged
    Unchanged,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Modified => write!(f, "modified"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}

/// Diff mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiffMode {
    /// Show all differences
    #[default]
    All,
    /// Only additions
    AdditionsOnly,
    /// Only removals
    RemovalsOnly,
    /// Only modifications
    ModificationsOnly,
}

impl std::fmt::Display for DiffMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::AdditionsOnly => write!(f, "additions_only"),
            Self::RemovalsOnly => write!(f, "removals_only"),
            Self::ModificationsOnly => write!(f, "modifications_only"),
        }
    }
}
