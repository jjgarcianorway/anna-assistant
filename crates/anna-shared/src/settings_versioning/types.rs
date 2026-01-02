// v0.0.588: Settings Versioning Types (Phase 164)
// Core types for settings version control

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Change type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Value added
    Added,
    /// Value modified
    Modified,
    /// Value removed
    Removed,
    /// Category reset
    Reset,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Modified => write!(f, "modified"),
            Self::Removed => write!(f, "removed"),
            Self::Reset => write!(f, "reset"),
        }
    }
}

/// Single change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    /// Change type
    pub change_type: ChangeType,
    /// Category
    pub category: SettingsCategory,
    /// Key path
    pub key: String,
    /// Previous value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
}

impl VersionChange {
    /// Create added change
    pub fn added(category: SettingsCategory, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::Added,
            category,
            key: key.into(),
            old_value: None,
            new_value: Some(value.into()),
        }
    }

    /// Create modified change
    pub fn modified(
        category: SettingsCategory,
        key: impl Into<String>,
        old: impl Into<String>,
        new: impl Into<String>,
    ) -> Self {
        Self {
            change_type: ChangeType::Modified,
            category,
            key: key.into(),
            old_value: Some(old.into()),
            new_value: Some(new.into()),
        }
    }

    /// Create removed change
    pub fn removed(category: SettingsCategory, key: impl Into<String>, old: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::Removed,
            category,
            key: key.into(),
            old_value: Some(old.into()),
            new_value: None,
        }
    }
}

/// Settings version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsVersion {
    /// Version number
    pub version: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Author (user or system)
    pub author: String,
    /// Description
    pub message: String,
    /// Changes in this version
    pub changes: Vec<VersionChange>,
    /// Snapshot hash
    pub hash: String,
    /// Parent version
    pub parent: Option<u64>,
}

impl SettingsVersion {
    /// Create new version
    pub fn new(version: u64, message: impl Into<String>) -> Self {
        Self {
            version,
            timestamp: chrono::Utc::now(),
            author: "user".to_string(),
            message: message.into(),
            changes: Vec::new(),
            hash: uuid::Uuid::new_v4().to_string(),
            parent: if version > 1 { Some(version - 1) } else { None },
        }
    }

    /// Set author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Add change
    pub fn add_change(&mut self, change: VersionChange) {
        self.changes.push(change);
    }

    /// Change count
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    /// Has changes
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Changes by category
    pub fn changes_for(&self, category: SettingsCategory) -> Vec<&VersionChange> {
        self.changes.iter().filter(|c| c.category == category).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_display() {
        assert_eq!(format!("{}", ChangeType::Added), "added");
        assert_eq!(format!("{}", ChangeType::Modified), "modified");
    }

    #[test]
    fn test_version_change_added() {
        let change = VersionChange::added(SettingsCategory::Personality, "key", "value");
        assert_eq!(change.change_type, ChangeType::Added);
        assert!(change.new_value.is_some());
    }

    #[test]
    fn test_version_change_modified() {
        let change = VersionChange::modified(SettingsCategory::Risk, "level", "low", "high");
        assert_eq!(change.change_type, ChangeType::Modified);
        assert!(change.old_value.is_some());
        assert!(change.new_value.is_some());
    }

    #[test]
    fn test_settings_version_new() {
        let version = SettingsVersion::new(1, "Initial version");
        assert_eq!(version.version, 1);
        assert_eq!(version.message, "Initial version");
    }

    #[test]
    fn test_settings_version_add_change() {
        let mut version = SettingsVersion::new(1, "Test");
        version.add_change(VersionChange::added(SettingsCategory::Learning, "k", "v"));
        assert_eq!(version.change_count(), 1);
    }
}
