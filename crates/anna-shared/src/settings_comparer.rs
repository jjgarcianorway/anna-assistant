// v0.0.689: Settings Comparer (Phase 265)
// Compare two settings collections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Comparer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparerConfig {
    /// Compare mode
    pub mode: CompareMode,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Include unchanged
    pub include_unchanged: bool,
    /// Ignore whitespace
    pub ignore_whitespace: bool,
}

impl ComparerConfig {
    /// Create new config
    pub fn new(mode: CompareMode) -> Self {
        Self {
            mode,
            case_insensitive: false,
            include_unchanged: false,
            ignore_whitespace: false,
        }
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set include unchanged
    pub fn include_unchanged(mut self, include: bool) -> Self {
        self.include_unchanged = include;
        self
    }

    /// Set ignore whitespace
    pub fn ignore_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_whitespace = ignore;
        self
    }
}

impl Default for ComparerConfig {
    fn default() -> Self {
        Self::new(CompareMode::Full)
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

/// Compare result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    /// Diff entries
    pub entries: Vec<DiffEntry>,
    /// Total left
    pub total_left: usize,
    /// Total right
    pub total_right: usize,
    /// Added count
    pub added: usize,
    /// Removed count
    pub removed: usize,
    /// Changed count
    pub changed: usize,
    /// Unchanged count
    pub unchanged: usize,
}

impl CompareResult {
    /// Create new result
    pub fn new(entries: Vec<DiffEntry>, left: usize, right: usize) -> Self {
        let added = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Added)).count();
        let removed = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Removed)).count();
        let changed = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Changed)).count();
        let unchanged = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Unchanged)).count();

        Self {
            entries,
            total_left: left,
            total_right: right,
            added,
            removed,
            changed,
            unchanged,
        }
    }

    /// Has changes
    pub fn has_changes(&self) -> bool {
        self.added > 0 || self.removed > 0 || self.changed > 0
    }

    /// Are identical
    pub fn are_identical(&self) -> bool {
        !self.has_changes()
    }

    /// Filter by type
    pub fn filter_by_type(&self, diff_type: DiffType) -> Vec<&DiffEntry> {
        self.entries.iter().filter(|e| e.diff_type == diff_type).collect()
    }

    /// Change summary
    pub fn summary(&self) -> String {
        format!("+{} -{} ~{}", self.added, self.removed, self.changed)
    }
}

impl Default for CompareResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0)
    }
}

/// Comparer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparerStats {
    /// Total comparisons
    pub total_comparisons: usize,
    /// Total entries compared
    pub total_entries: usize,
    /// Total changes found
    pub total_changes: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ComparerStats {
    /// Record comparison
    pub fn record(&mut self, result: &CompareResult, mode: CompareMode) {
        self.total_comparisons += 1;
        self.total_entries += result.total_left + result.total_right;
        self.total_changes += result.added + result.removed + result.changed;
        *self.by_mode.entry(mode.to_string()).or_insert(0) += 1;
    }

    /// Avg changes per comparison
    pub fn avg_changes(&self) -> f64 {
        if self.total_comparisons == 0 {
            0.0
        } else {
            self.total_changes as f64 / self.total_comparisons as f64
        }
    }
}

/// Settings comparer
#[derive(Debug, Clone, Default)]
pub struct SettingsComparer {
    /// Config
    config: ComparerConfig,
    /// Stats
    stats: ComparerStats,
}

impl SettingsComparer {
    /// Create new comparer
    pub fn new(config: ComparerConfig) -> Self {
        Self {
            config,
            stats: ComparerStats::default(),
        }
    }

    /// Normalize value for comparison
    fn normalize(&self, value: &str) -> String {
        let mut v = if self.config.case_insensitive {
            value.to_lowercase()
        } else {
            value.to_string()
        };

        if self.config.ignore_whitespace {
            v = v.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        v
    }

    /// Compare two settings
    pub fn compare(&mut self, left: &HashMap<String, String>, right: &HashMap<String, String>) -> CompareResult {
        let mut entries = Vec::new();

        // Check left side
        for (key, left_value) in left {
            if let Some(right_value) = right.get(key) {
                let left_norm = self.normalize(left_value);
                let right_norm = self.normalize(right_value);

                if left_norm == right_norm {
                    if self.config.include_unchanged {
                        entries.push(DiffEntry::new(key.clone(), Some(left_value.clone()), Some(right_value.clone()), DiffType::Unchanged));
                    }
                } else {
                    entries.push(DiffEntry::new(key.clone(), Some(left_value.clone()), Some(right_value.clone()), DiffType::Changed));
                }
            } else {
                entries.push(DiffEntry::new(key.clone(), Some(left_value.clone()), None, DiffType::Removed));
            }
        }

        // Check for additions (keys in right but not in left)
        for (key, right_value) in right {
            if !left.contains_key(key) {
                entries.push(DiffEntry::new(key.clone(), None, Some(right_value.clone()), DiffType::Added));
            }
        }

        // Sort by key
        entries.sort_by(|a, b| a.key.cmp(&b.key));

        let result = CompareResult::new(entries, left.len(), right.len());
        self.stats.record(&result, self.config.mode);
        result
    }

    /// Compare keys only
    pub fn compare_keys(&mut self, left: &HashMap<String, String>, right: &HashMap<String, String>) -> CompareResult {
        let mut entries = Vec::new();

        for key in left.keys() {
            if right.contains_key(key) {
                if self.config.include_unchanged {
                    entries.push(DiffEntry::new(key.clone(), None, None, DiffType::Unchanged));
                }
            } else {
                entries.push(DiffEntry::new(key.clone(), None, None, DiffType::Removed));
            }
        }

        for key in right.keys() {
            if !left.contains_key(key) {
                entries.push(DiffEntry::new(key.clone(), None, None, DiffType::Added));
            }
        }

        entries.sort_by(|a, b| a.key.cmp(&b.key));

        let result = CompareResult::new(entries, left.len(), right.len());
        self.stats.record(&result, CompareMode::KeysOnly);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ComparerStats {
        &self.stats
    }
}

/// Comparer registry
#[derive(Debug, Clone, Default)]
pub struct ComparerRegistry {
    /// Comparers by ID
    comparers: HashMap<String, SettingsComparer>,
}

impl ComparerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register comparer
    pub fn register(&mut self, id: impl Into<String>, comparer: SettingsComparer) {
        self.comparers.insert(id.into(), comparer);
    }

    /// Unregister comparer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.comparers.remove(id).is_some()
    }

    /// Get comparer
    pub fn get(&self, id: &str) -> Option<&SettingsComparer> {
        self.comparers.get(id)
    }

    /// Get comparer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsComparer> {
        self.comparers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.comparers.len()
    }
}

/// Format comparer registry
pub fn format_comparer_registry(registry: &ComparerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Comparer Registry:\n");
    output.push_str(&format!("  Comparers: {}\n", registry.count()));
    output
}

/// Check if query is about comparer
pub fn is_comparer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("compare settings") || lower.contains("settings comparer") || lower.contains("diff settings")
}

/// Fun fact about comparer
pub fn comparer_fun_fact() -> &'static str {
    "Anna's settings comparer shows exactly what changed between configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_mode_display() {
        assert_eq!(format!("{}", CompareMode::Full), "full");
        assert_eq!(format!("{}", CompareMode::KeysOnly), "keys_only");
    }

    #[test]
    fn test_diff_type_display() {
        assert_eq!(format!("{}", DiffType::Added), "added");
        assert_eq!(format!("{}", DiffType::Removed), "removed");
    }

    #[test]
    fn test_config_new() {
        let c = ComparerConfig::new(CompareMode::Full);
        assert_eq!(c.mode, CompareMode::Full);
    }

    #[test]
    fn test_config_builder() {
        let c = ComparerConfig::new(CompareMode::Full)
            .case_insensitive(true)
            .include_unchanged(true);
        assert!(c.case_insensitive);
        assert!(c.include_unchanged);
    }

    #[test]
    fn test_diff_entry_new() {
        let e = DiffEntry::new("key", Some("old".to_string()), Some("new".to_string()), DiffType::Changed);
        assert!(e.is_change());
        assert!(e.value_changed());
    }

    #[test]
    fn test_diff_entry_unchanged() {
        let e = DiffEntry::new("key", Some("val".to_string()), Some("val".to_string()), DiffType::Unchanged);
        assert!(!e.is_change());
    }

    #[test]
    fn test_result_new() {
        let entries = vec![
            DiffEntry::new("k1", None, Some("v".to_string()), DiffType::Added),
            DiffEntry::new("k2", Some("v".to_string()), None, DiffType::Removed),
        ];
        let r = CompareResult::new(entries, 2, 2);
        assert_eq!(r.added, 1);
        assert_eq!(r.removed, 1);
    }

    #[test]
    fn test_result_has_changes() {
        let r = CompareResult::new(vec![DiffEntry::new("k", None, Some("v".to_string()), DiffType::Added)], 0, 1);
        assert!(r.has_changes());
    }

    #[test]
    fn test_result_identical() {
        let r = CompareResult::new(Vec::new(), 0, 0);
        assert!(r.are_identical());
    }

    #[test]
    fn test_result_summary() {
        let entries = vec![DiffEntry::new("k", None, Some("v".to_string()), DiffType::Added)];
        let r = CompareResult::new(entries, 0, 1);
        assert_eq!(r.summary(), "+1 -0 ~0");
    }

    #[test]
    fn test_stats_record() {
        let mut s = ComparerStats::default();
        let r = CompareResult::new(Vec::new(), 5, 5);
        s.record(&r, CompareMode::Full);
        assert_eq!(s.total_comparisons, 1);
    }

    #[test]
    fn test_comparer_new() {
        let c = SettingsComparer::new(ComparerConfig::default());
        assert_eq!(c.stats().total_comparisons, 0);
    }

    #[test]
    fn test_comparer_compare_identical() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let mut left = HashMap::new();
        left.insert("key".to_string(), "value".to_string());
        let right = left.clone();

        let result = c.compare(&left, &right);
        assert!(result.are_identical());
    }

    #[test]
    fn test_comparer_compare_added() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let left = HashMap::new();
        let mut right = HashMap::new();
        right.insert("key".to_string(), "value".to_string());

        let result = c.compare(&left, &right);
        assert_eq!(result.added, 1);
    }

    #[test]
    fn test_comparer_compare_removed() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let mut left = HashMap::new();
        left.insert("key".to_string(), "value".to_string());
        let right = HashMap::new();

        let result = c.compare(&left, &right);
        assert_eq!(result.removed, 1);
    }

    #[test]
    fn test_comparer_compare_changed() {
        let mut c = SettingsComparer::new(ComparerConfig::default());
        let mut left = HashMap::new();
        left.insert("key".to_string(), "old".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "new".to_string());

        let result = c.compare(&left, &right);
        assert_eq!(result.changed, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ComparerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ComparerRegistry::new();
        r.register("c1", SettingsComparer::new(ComparerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_comparer_query() {
        assert!(is_comparer_query("compare settings"));
        assert!(!is_comparer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = comparer_fun_fact();
        assert!(fact.contains("comparer"));
    }
}
