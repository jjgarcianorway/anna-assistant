// v0.0.689: Settings Comparer Types (Phase 265)
// Core types for settings comparison

use serde::{Deserialize, Serialize};

/// Compare mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompareMode {
    /// Full comparison
    #[default]
    Full,
    /// Keys only
    KeysOnly,
    /// Values only
    ValuesOnly,
    /// Structure only
    StructureOnly,
}

impl std::fmt::Display for CompareMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::KeysOnly => write!(f, "keys_only"),
            Self::ValuesOnly => write!(f, "values_only"),
            Self::StructureOnly => write!(f, "structure_only"),
        }
    }
}

/// Difference type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiffType {
    /// Added
    #[default]
    Added,
    /// Removed
    Removed,
    /// Changed
    Changed,
    /// Unchanged
    Unchanged,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Changed => write!(f, "changed"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}

/// Diff entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Key
    pub key: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Diff type
    pub diff_type: DiffType,
}

impl DiffEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, old: Option<String>, new: Option<String>, diff_type: DiffType) -> Self {
        Self {
            key: key.into(),
            old_value: old,
            new_value: new,
            diff_type,
        }
    }

    /// Is change
    pub fn is_change(&self) -> bool {
        !matches!(self.diff_type, DiffType::Unchanged)
    }

    /// Value changed
    pub fn value_changed(&self) -> bool {
        matches!(self.diff_type, DiffType::Changed)
    }
}
