// Types for settings diff

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Type of change in a diff
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// Value was added
    Added,
    /// Value was removed
    Removed,
    /// Value was changed
    Changed,
    /// No change
    Unchanged,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "+"),
            Self::Removed => write!(f, "-"),
            Self::Changed => write!(f, "~"),
            Self::Unchanged => write!(f, " "),
        }
    }
}

/// A single difference entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Category affected
    pub category: SettingsCategory,
    /// Field path within category
    pub field: String,
    /// Type of change
    pub diff_type: DiffType,
    /// Old value (serialized)
    pub old_value: Option<String>,
    /// New value (serialized)
    pub new_value: Option<String>,
}

impl DiffEntry {
    /// Create a new diff entry
    pub fn new(
        category: SettingsCategory,
        field: impl Into<String>,
        diff_type: DiffType,
    ) -> Self {
        Self {
            category,
            field: field.into(),
            diff_type,
            old_value: None,
            new_value: None,
        }
    }

    /// Set old value
    pub fn old(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set new value
    pub fn new_val(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// Is this a change?
    pub fn is_changed(&self) -> bool {
        self.diff_type != DiffType::Unchanged
    }
}

/// Result of comparing two settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsDiff {
    /// All differences found
    pub entries: Vec<DiffEntry>,
    /// Categories that changed
    pub changed_categories: Vec<SettingsCategory>,
}

impl SettingsDiff {
    /// Create empty diff result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diff entry
    pub fn add(&mut self, entry: DiffEntry) {
        if entry.is_changed() && !self.changed_categories.contains(&entry.category) {
            self.changed_categories.push(entry.category);
        }
        self.entries.push(entry);
    }

    /// Are the settings identical?
    pub fn is_identical(&self) -> bool {
        self.entries.iter().all(|e| !e.is_changed())
    }

    /// Has any changes?
    pub fn has_changes(&self) -> bool {
        self.entries.iter().any(|e| e.is_changed())
    }

    /// Get only changes (filter out unchanged)
    pub fn changes_only(&self) -> Vec<&DiffEntry> {
        self.entries.iter().filter(|e| e.is_changed()).collect()
    }

    /// Count changes
    pub fn change_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_changed()).count()
    }

    /// Get changes for a specific category
    pub fn category_changes(&self, category: SettingsCategory) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category && e.is_changed())
            .collect()
    }
}
