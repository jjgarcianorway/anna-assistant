// v0.0.601: Settings Comparator Types (Phase 177)
// Type definitions for settings comparison

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Difference type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// Added
    Added,
    /// Removed
    Removed,
    /// Modified
    Modified,
    /// Unchanged
    Unchanged,
    /// Type changed
    TypeChanged,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Modified => write!(f, "modified"),
            Self::Unchanged => write!(f, "unchanged"),
            Self::TypeChanged => write!(f, "type_changed"),
        }
    }
}

/// Comparison mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareMode {
    /// Full comparison
    Full,
    /// Changes only
    ChangesOnly,
    /// Additions only
    AdditionsOnly,
    /// Removals only
    RemovalsOnly,
    /// Summary only
    SummaryOnly,
}

impl std::fmt::Display for CompareMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::ChangesOnly => write!(f, "changes_only"),
            Self::AdditionsOnly => write!(f, "additions_only"),
            Self::RemovalsOnly => write!(f, "removals_only"),
            Self::SummaryOnly => write!(f, "summary_only"),
        }
    }
}

/// Single difference entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Key
    pub key: String,
    /// Category
    pub category: SettingsCategory,
    /// Difference type
    pub diff_type: DiffType,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
}

impl DiffEntry {
    /// Create added entry
    pub fn added(key: impl Into<String>, category: SettingsCategory, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            category,
            diff_type: DiffType::Added,
            old_value: None,
            new_value: Some(value.into()),
        }
    }

    /// Create removed entry
    pub fn removed(key: impl Into<String>, category: SettingsCategory, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            category,
            diff_type: DiffType::Removed,
            old_value: Some(value.into()),
            new_value: None,
        }
    }

    /// Create modified entry
    pub fn modified(
        key: impl Into<String>,
        category: SettingsCategory,
        old: impl Into<String>,
        new: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            category,
            diff_type: DiffType::Modified,
            old_value: Some(old.into()),
            new_value: Some(new.into()),
        }
    }

    /// Is change (not unchanged)
    pub fn is_change(&self) -> bool {
        self.diff_type != DiffType::Unchanged
    }
}

/// Comparison result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompareResult {
    /// Source label
    pub source_label: String,
    /// Target label
    pub target_label: String,
    /// Differences
    pub diffs: Vec<DiffEntry>,
    /// Added count
    pub added: usize,
    /// Removed count
    pub removed: usize,
    /// Modified count
    pub modified: usize,
    /// Unchanged count
    pub unchanged: usize,
}

impl CompareResult {
    /// Create new result
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source_label: source.into(),
            target_label: target.into(),
            ..Default::default()
        }
    }

    /// Add diff entry
    pub fn add(&mut self, entry: DiffEntry) {
        match entry.diff_type {
            DiffType::Added => self.added += 1,
            DiffType::Removed => self.removed += 1,
            DiffType::Modified | DiffType::TypeChanged => self.modified += 1,
            DiffType::Unchanged => self.unchanged += 1,
        }
        self.diffs.push(entry);
    }

    /// Total differences
    pub fn total_changes(&self) -> usize {
        self.added + self.removed + self.modified
    }

    /// Has changes
    pub fn has_changes(&self) -> bool {
        self.total_changes() > 0
    }

    /// Get changes only
    pub fn changes_only(&self) -> Vec<&DiffEntry> {
        self.diffs.iter().filter(|d| d.is_change()).collect()
    }

    /// Get by category
    pub fn by_category(&self, category: SettingsCategory) -> Vec<&DiffEntry> {
        self.diffs.iter().filter(|d| d.category == category).collect()
    }
}

/// Comparison options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareOptions {
    /// Mode
    pub mode: CompareMode,
    /// Categories to include (empty = all)
    pub categories: Vec<SettingsCategory>,
    /// Keys to ignore
    pub ignore_keys: Vec<String>,
    /// Case sensitive
    pub case_sensitive: bool,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            mode: CompareMode::Full,
            categories: Vec::new(),
            ignore_keys: Vec::new(),
            case_sensitive: true,
        }
    }
}

impl CompareOptions {
    /// Create new options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set mode
    pub fn mode(mut self, mode: CompareMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add category filter
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Add ignore key
    pub fn ignore(mut self, key: impl Into<String>) -> Self {
        self.ignore_keys.push(key.into());
        self
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Should include category
    pub fn includes_category(&self, category: SettingsCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(&category)
    }

    /// Should ignore key
    pub fn should_ignore(&self, key: &str) -> bool {
        self.ignore_keys.iter().any(|k| k == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_type_display() {
        assert_eq!(format!("{}", DiffType::Added), "added");
        assert_eq!(format!("{}", DiffType::Modified), "modified");
    }

    #[test]
    fn test_compare_mode_display() {
        assert_eq!(format!("{}", CompareMode::Full), "full");
        assert_eq!(format!("{}", CompareMode::ChangesOnly), "changes_only");
    }

    #[test]
    fn test_diff_entry_added() {
        let e = DiffEntry::added("key", SettingsCategory::Personality, "value");
        assert_eq!(e.diff_type, DiffType::Added);
        assert!(e.is_change());
    }

    #[test]
    fn test_diff_entry_modified() {
        let e = DiffEntry::modified("key", SettingsCategory::Privacy, "old", "new");
        assert_eq!(e.diff_type, DiffType::Modified);
    }

    #[test]
    fn test_compare_result_new() {
        let r = CompareResult::new("v1", "v2");
        assert_eq!(r.source_label, "v1");
        assert!(!r.has_changes());
    }

    #[test]
    fn test_compare_result_add() {
        let mut r = CompareResult::new("a", "b");
        r.add(DiffEntry::added("k", SettingsCategory::Risk, "v"));
        assert_eq!(r.added, 1);
        assert!(r.has_changes());
    }

    #[test]
    fn test_compare_options_default() {
        let o = CompareOptions::new();
        assert_eq!(o.mode, CompareMode::Full);
        assert!(o.case_sensitive);
    }

    #[test]
    fn test_compare_options_builder() {
        let o = CompareOptions::new()
            .mode(CompareMode::ChangesOnly)
            .category(SettingsCategory::Personality)
            .ignore("secret");
        assert!(o.should_ignore("secret"));
    }
}
