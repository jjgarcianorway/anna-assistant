// v0.0.661: Settings Differ (Phase 237)
// Differ for comparing settings configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

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

/// Differ config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferConfig {
    /// Diff mode
    pub mode: DiffMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include unchanged
    pub include_unchanged: bool,
    /// Case sensitive comparison
    pub case_sensitive: bool,
}

impl DifferConfig {
    /// Create new config
    pub fn new(mode: DiffMode) -> Self {
        Self {
            mode,
            category: None,
            include_unchanged: false,
            case_sensitive: true,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include unchanged
    pub fn include_unchanged(mut self, include: bool) -> Self {
        self.include_unchanged = include;
        self
    }

    /// Set case sensitive
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }
}

impl Default for DifferConfig {
    fn default() -> Self {
        Self::new(DiffMode::All)
    }
}

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

/// Diff result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// All diff entries
    pub entries: Vec<DiffEntry>,
    /// Count by type
    pub added_count: usize,
    /// Removed count
    pub removed_count: usize,
    /// Modified count
    pub modified_count: usize,
    /// Unchanged count
    pub unchanged_count: usize,
}

impl DiffResult {
    /// Create new result
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            added_count: 0,
            removed_count: 0,
            modified_count: 0,
            unchanged_count: 0,
        }
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: DiffEntry) {
        match entry.diff_type {
            DiffType::Added => self.added_count += 1,
            DiffType::Removed => self.removed_count += 1,
            DiffType::Modified => self.modified_count += 1,
            DiffType::Unchanged => self.unchanged_count += 1,
        }
        self.entries.push(entry);
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.added_count + self.removed_count + self.modified_count
    }

    /// Has changes
    pub fn has_changes(&self) -> bool {
        self.total_changes() > 0
    }

    /// Get entries by type
    pub fn get_by_type(&self, diff_type: DiffType) -> Vec<&DiffEntry> {
        self.entries.iter().filter(|e| e.diff_type == diff_type).collect()
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Differ stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifferStats {
    /// Total diffs performed
    pub total_diffs: usize,
    /// Total changes found
    pub total_changes_found: usize,
    /// By diff type
    pub by_type: HashMap<String, usize>,
}

impl DifferStats {
    /// Record diff
    pub fn record(&mut self, added: usize, removed: usize, modified: usize) {
        self.total_diffs += 1;
        self.total_changes_found += added + removed + modified;
        *self.by_type.entry("added".to_string()).or_insert(0) += added;
        *self.by_type.entry("removed".to_string()).or_insert(0) += removed;
        *self.by_type.entry("modified".to_string()).or_insert(0) += modified;
    }

    /// Average changes per diff
    pub fn average_changes(&self) -> f64 {
        if self.total_diffs == 0 {
            0.0
        } else {
            self.total_changes_found as f64 / self.total_diffs as f64
        }
    }
}

/// Settings differ
#[derive(Debug, Clone, Default)]
pub struct SettingsDiffer {
    /// Config
    config: DifferConfig,
    /// Results
    results: Vec<DiffResult>,
    /// Stats
    stats: DifferStats,
}

impl SettingsDiffer {
    /// Create new differ
    pub fn new(config: DifferConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: DifferStats::default(),
        }
    }

    /// Compare two settings maps
    pub fn diff(
        &mut self,
        old: &HashMap<String, String>,
        new: &HashMap<String, String>,
    ) -> DiffResult {
        let mut result = DiffResult::new();

        // Check for removed and modified
        for (key, old_value) in old {
            if let Some(new_value) = new.get(key) {
                let values_equal = if self.config.case_sensitive {
                    old_value == new_value
                } else {
                    old_value.to_lowercase() == new_value.to_lowercase()
                };

                if values_equal {
                    if self.config.include_unchanged {
                        result.add_entry(DiffEntry::unchanged(key, old_value));
                    }
                } else if self.should_include(DiffType::Modified) {
                    result.add_entry(DiffEntry::modified(key, old_value, new_value));
                }
            } else if self.should_include(DiffType::Removed) {
                result.add_entry(DiffEntry::removed(key, old_value));
            }
        }

        // Check for added
        for (key, new_value) in new {
            if !old.contains_key(key) && self.should_include(DiffType::Added) {
                result.add_entry(DiffEntry::added(key, new_value));
            }
        }

        self.stats.record(
            result.added_count,
            result.removed_count,
            result.modified_count,
        );
        self.results.push(result.clone());
        result
    }

    /// Check if diff type should be included
    fn should_include(&self, diff_type: DiffType) -> bool {
        match self.config.mode {
            DiffMode::All => true,
            DiffMode::AdditionsOnly => diff_type == DiffType::Added,
            DiffMode::RemovalsOnly => diff_type == DiffType::Removed,
            DiffMode::ModificationsOnly => diff_type == DiffType::Modified,
        }
    }

    /// Get results
    pub fn results(&self) -> &[DiffResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &DifferStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings differ registry
#[derive(Debug, Clone, Default)]
pub struct SettingsDifferRegistry {
    /// Differs by ID
    differs: HashMap<String, SettingsDiffer>,
}

impl SettingsDifferRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register differ
    pub fn register(&mut self, id: impl Into<String>, differ: SettingsDiffer) {
        self.differs.insert(id.into(), differ);
    }

    /// Unregister differ
    pub fn unregister(&mut self, id: &str) -> bool {
        self.differs.remove(id).is_some()
    }

    /// Get differ
    pub fn get(&self, id: &str) -> Option<&SettingsDiffer> {
        self.differs.get(id)
    }

    /// Get differ mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDiffer> {
        self.differs.get_mut(id)
    }

    /// Differ count
    pub fn count(&self) -> usize {
        self.differs.len()
    }
}

/// Format differ registry
pub fn format_differ_registry(registry: &SettingsDifferRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Differ Registry:\n");
    output.push_str(&format!("  Differs: {}\n", registry.count()));
    output
}

/// Check if query is about differ
pub fn is_differ_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("differ") || lower.contains("diff settings") || lower.contains("compare settings")
}

/// Fun fact about differ
pub fn differ_fun_fact() -> &'static str {
    "Anna's settings differs spot every config change!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_type_display() {
        assert_eq!(format!("{}", DiffType::Added), "added");
        assert_eq!(format!("{}", DiffType::Removed), "removed");
    }

    #[test]
    fn test_diff_mode_display() {
        assert_eq!(format!("{}", DiffMode::All), "all");
        assert_eq!(format!("{}", DiffMode::AdditionsOnly), "additions_only");
    }

    #[test]
    fn test_config_new() {
        let c = DifferConfig::new(DiffMode::All);
        assert!(c.case_sensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = DifferConfig::new(DiffMode::ModificationsOnly)
            .include_unchanged(true)
            .case_sensitive(false);
        assert!(c.include_unchanged);
        assert!(!c.case_sensitive);
    }

    #[test]
    fn test_entry_added() {
        let e = DiffEntry::added("key", "value");
        assert_eq!(e.diff_type, DiffType::Added);
        assert!(e.old_value.is_none());
    }

    #[test]
    fn test_entry_removed() {
        let e = DiffEntry::removed("key", "value");
        assert_eq!(e.diff_type, DiffType::Removed);
        assert!(e.new_value.is_none());
    }

    #[test]
    fn test_entry_modified() {
        let e = DiffEntry::modified("key", "old", "new");
        assert_eq!(e.diff_type, DiffType::Modified);
        assert_eq!(e.old_value, Some("old".to_string()));
        assert_eq!(e.new_value, Some("new".to_string()));
    }

    #[test]
    fn test_result_new() {
        let r = DiffResult::new();
        assert_eq!(r.total_changes(), 0);
    }

    #[test]
    fn test_result_add_entry() {
        let mut r = DiffResult::new();
        r.add_entry(DiffEntry::added("key", "value"));
        assert_eq!(r.added_count, 1);
        assert_eq!(r.total_changes(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DifferStats::default();
        s.record(5, 3, 2);
        assert_eq!(s.total_diffs, 1);
        assert_eq!(s.total_changes_found, 10);
    }

    #[test]
    fn test_differ_new() {
        let d = SettingsDiffer::new(DifferConfig::new(DiffMode::All));
        assert_eq!(d.result_count(), 0);
    }

    #[test]
    fn test_differ_diff_no_changes() {
        let mut d = SettingsDiffer::new(DifferConfig::new(DiffMode::All));
        let mut old = HashMap::new();
        old.insert("key".to_string(), "value".to_string());
        let new = old.clone();

        let r = d.diff(&old, &new);
        assert!(!r.has_changes());
    }

    #[test]
    fn test_differ_diff_with_changes() {
        let mut d = SettingsDiffer::new(DifferConfig::new(DiffMode::All));
        let mut old = HashMap::new();
        old.insert("key1".to_string(), "value1".to_string());
        old.insert("key2".to_string(), "old_value".to_string());

        let mut new = HashMap::new();
        new.insert("key2".to_string(), "new_value".to_string());
        new.insert("key3".to_string(), "value3".to_string());

        let r = d.diff(&old, &new);
        assert_eq!(r.removed_count, 1); // key1 removed
        assert_eq!(r.modified_count, 1); // key2 modified
        assert_eq!(r.added_count, 1); // key3 added
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsDifferRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsDifferRegistry::new();
        r.register("d1", SettingsDiffer::new(DifferConfig::new(DiffMode::All)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_differ_query() {
        assert!(is_differ_query("settings differ"));
        assert!(!is_differ_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = differ_fun_fact();
        assert!(fact.contains("differ"));
    }
}
