// v0.0.661: Settings Differ Entry (Phase 237)
// Diff entry representation

use serde::{Deserialize, Serialize};

use super::types::DiffType;

/// Single diff entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Key
    pub key: String,
    /// Diff type
    pub diff_type: DiffType,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
}

impl DiffEntry {
    /// Create added entry
    pub fn added(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            diff_type: DiffType::Added,
            old_value: None,
            new_value: Some(value.into()),
        }
    }

    /// Create removed entry
    pub fn removed(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            diff_type: DiffType::Removed,
            old_value: Some(value.into()),
            new_value: None,
        }
    }

    /// Create modified entry
    pub fn modified(key: impl Into<String>, old: impl Into<String>, new: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            diff_type: DiffType::Modified,
            old_value: Some(old.into()),
            new_value: Some(new.into()),
        }
    }

    /// Create unchanged entry
    pub fn unchanged(key: impl Into<String>, value: impl Into<String>) -> Self {
        let v = value.into();
        Self {
            key: key.into(),
            diff_type: DiffType::Unchanged,
            old_value: Some(v.clone()),
            new_value: Some(v),
        }
    }
}
