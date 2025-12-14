// v0.0.588: Settings Versioning (Phase 164)
// Version control for settings with history and comparison

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

/// Version history
#[derive(Debug, Clone, Default)]
pub struct VersionHistory {
    /// All versions
    versions: Vec<SettingsVersion>,
    /// Current version
    current: u64,
    /// Max versions to keep
    max_versions: usize,
}

impl VersionHistory {
    /// Create new history
    pub fn new() -> Self {
        Self {
            max_versions: 100,
            ..Default::default()
        }
    }

    /// Set max versions
    pub fn max_versions(mut self, max: usize) -> Self {
        self.max_versions = max;
        self
    }

    /// Create new version
    pub fn create(&mut self, message: impl Into<String>) -> &mut SettingsVersion {
        self.current += 1;
        let version = SettingsVersion::new(self.current, message);
        self.versions.push(version);

        while self.versions.len() > self.max_versions {
            self.versions.remove(0);
        }

        self.versions.last_mut().unwrap()
    }

    /// Get current version
    pub fn current(&self) -> Option<&SettingsVersion> {
        self.versions.last()
    }

    /// Get version by number
    pub fn get(&self, version: u64) -> Option<&SettingsVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Get all versions
    pub fn all(&self) -> &[SettingsVersion] {
        &self.versions
    }

    /// Get recent versions
    pub fn recent(&self, count: usize) -> Vec<&SettingsVersion> {
        self.versions.iter().rev().take(count).collect()
    }

    /// Version count
    pub fn count(&self) -> usize {
        self.versions.len()
    }

    /// Current version number
    pub fn current_version(&self) -> u64 {
        self.current
    }

    /// Compare two versions
    pub fn compare(&self, v1: u64, v2: u64) -> Option<Vec<&VersionChange>> {
        let ver1 = self.get(v1)?;
        let ver2 = self.get(v2)?;

        // Get all changes between v1 and v2
        let min = v1.min(v2);
        let max = v1.max(v2);

        let changes: Vec<_> = self.versions
            .iter()
            .filter(|v| v.version > min && v.version <= max)
            .flat_map(|v| &v.changes)
            .collect();

        Some(changes)
    }

    /// Find version by message
    pub fn find_by_message(&self, search: &str) -> Vec<&SettingsVersion> {
        let lower = search.to_lowercase();
        self.versions
            .iter()
            .filter(|v| v.message.to_lowercase().contains(&lower))
            .collect()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.versions.clear();
        self.current = 0;
    }
}

/// Format version
pub fn format_version(version: &SettingsVersion) -> String {
    let mut output = String::new();

    output.push_str(&format!("Version {} - {}\n", version.version, version.message));
    output.push_str(&format!(
        "  {} | {} changes\n",
        version.timestamp.format("%Y-%m-%d %H:%M"),
        version.change_count()
    ));

    for change in &version.changes {
        output.push_str(&format!(
            "  {} {} {}\n",
            change.change_type, change.category, change.key
        ));
    }

    output
}

/// Format history summary
pub fn format_history(history: &VersionHistory) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Version History ===\n\n");
    output.push_str(&format!(
        "Current: v{} | Total: {} versions\n\n",
        history.current_version(),
        history.count()
    ));

    for version in history.recent(10) {
        output.push_str(&format!(
            "v{}: {} ({} changes)\n",
            version.version,
            version.message,
            version.change_count()
        ));
    }

    output
}

/// Check if query is about versioning
pub fn is_versioning_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("version")
        || lower.contains("history")
        || lower.contains("changelog")
        || lower.contains("compare")
}

/// Fun fact about versioning
pub fn settings_versioning_fun_fact() -> &'static str {
    "Anna tracks every settings change so you can always see what changed and when!"
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

    #[test]
    fn test_version_history_new() {
        let history = VersionHistory::new();
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_version_history_create() {
        let mut history = VersionHistory::new();
        history.create("First version");
        assert_eq!(history.count(), 1);
        assert_eq!(history.current_version(), 1);
    }

    #[test]
    fn test_version_history_get() {
        let mut history = VersionHistory::new();
        history.create("Version 1");
        history.create("Version 2");
        assert!(history.get(1).is_some());
        assert!(history.get(2).is_some());
        assert!(history.get(3).is_none());
    }

    #[test]
    fn test_version_history_recent() {
        let mut history = VersionHistory::new();
        history.create("V1");
        history.create("V2");
        history.create("V3");
        let recent = history.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].version, 3);
    }

    #[test]
    fn test_format_version() {
        let version = SettingsVersion::new(1, "Test version");
        let output = format_version(&version);
        assert!(output.contains("Version 1"));
    }

    #[test]
    fn test_format_history() {
        let history = VersionHistory::new();
        let output = format_history(&history);
        assert!(output.contains("History"));
    }

    #[test]
    fn test_is_versioning_query() {
        assert!(is_versioning_query("show version history"));
        assert!(is_versioning_query("compare versions"));
        assert!(!is_versioning_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_versioning_fun_fact();
        assert!(fact.contains("change"));
    }
}
