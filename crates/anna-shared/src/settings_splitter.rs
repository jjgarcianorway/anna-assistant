// v0.0.656: Settings Splitter (Phase 232)
// Splitter for dividing settings into separate groups

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Split criteria
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SplitCriteria {
    /// By category
    #[default]
    ByCategory,
    /// By prefix
    ByPrefix,
    /// By pattern
    ByPattern,
    /// By value type
    ByValueType,
    /// By size
    BySize,
}

impl std::fmt::Display for SplitCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByCategory => write!(f, "by_category"),
            Self::ByPrefix => write!(f, "by_prefix"),
            Self::ByPattern => write!(f, "by_pattern"),
            Self::ByValueType => write!(f, "by_value_type"),
            Self::BySize => write!(f, "by_size"),
        }
    }
}

/// Split mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SplitMode {
    /// Even distribution
    #[default]
    Even,
    /// By threshold
    ByThreshold,
    /// By count
    ByCount,
    /// Custom
    Custom,
}

impl std::fmt::Display for SplitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Even => write!(f, "even"),
            Self::ByThreshold => write!(f, "by_threshold"),
            Self::ByCount => write!(f, "by_count"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Splitter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitterConfig {
    /// Split criteria
    pub criteria: SplitCriteria,
    /// Split mode
    pub mode: SplitMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Max groups
    pub max_groups: usize,
    /// Preserve order
    pub preserve_order: bool,
}

impl SplitterConfig {
    /// Create new config
    pub fn new(criteria: SplitCriteria) -> Self {
        Self {
            criteria,
            mode: SplitMode::Even,
            category: None,
            max_groups: 10,
            preserve_order: true,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: SplitMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set max groups
    pub fn max_groups(mut self, max: usize) -> Self {
        self.max_groups = max;
        self
    }

    /// Set preserve order
    pub fn preserve_order(mut self, preserve: bool) -> Self {
        self.preserve_order = preserve;
        self
    }
}

impl Default for SplitterConfig {
    fn default() -> Self {
        Self::new(SplitCriteria::ByCategory)
    }
}

/// Split group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitGroup {
    /// Group name
    pub name: String,
    /// Settings in this group
    pub settings: HashMap<String, String>,
    /// Criteria value
    pub criteria_value: String,
}

impl SplitGroup {
    /// Create new group
    pub fn new(name: impl Into<String>, criteria_value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            settings: HashMap::new(),
            criteria_value: criteria_value.into(),
        }
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Setting count
    pub fn setting_count(&self) -> usize {
        self.settings.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.settings.is_empty()
    }
}

/// Split result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitResult {
    /// Groups created
    pub groups: Vec<SplitGroup>,
    /// Total keys split
    pub total_keys: usize,
    /// Criteria used
    pub criteria: SplitCriteria,
    /// Unmatched keys
    pub unmatched: Vec<String>,
}

impl SplitResult {
    /// Create new result
    pub fn new(criteria: SplitCriteria) -> Self {
        Self {
            groups: Vec::new(),
            total_keys: 0,
            criteria,
            unmatched: Vec::new(),
        }
    }

    /// Add group
    pub fn add_group(&mut self, group: SplitGroup) {
        self.total_keys += group.setting_count();
        self.groups.push(group);
    }

    /// Add unmatched
    pub fn add_unmatched(&mut self, key: String) {
        self.unmatched.push(key);
    }

    /// Group count
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Has unmatched
    pub fn has_unmatched(&self) -> bool {
        !self.unmatched.is_empty()
    }
}

/// Splitter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SplitterStats {
    /// Total splits
    pub total_splits: usize,
    /// Total groups created
    pub total_groups: usize,
    /// Total keys split
    pub total_keys_split: usize,
    /// By criteria
    pub by_criteria: HashMap<String, usize>,
}

impl SplitterStats {
    /// Record split
    pub fn record(&mut self, criteria: SplitCriteria, groups: usize, keys: usize) {
        self.total_splits += 1;
        self.total_groups += groups;
        self.total_keys_split += keys;
        *self.by_criteria.entry(criteria.to_string()).or_insert(0) += 1;
    }

    /// Average group size
    pub fn average_group_size(&self) -> f64 {
        if self.total_groups == 0 {
            0.0
        } else {
            self.total_keys_split as f64 / self.total_groups as f64
        }
    }
}

/// Settings splitter
#[derive(Debug, Clone, Default)]
pub struct SettingsSplitter {
    /// Config
    config: SplitterConfig,
    /// Results
    results: Vec<SplitResult>,
    /// Stats
    stats: SplitterStats,
}

impl SettingsSplitter {
    /// Create new splitter
    pub fn new(config: SplitterConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: SplitterStats::default(),
        }
    }

    /// Split settings
    pub fn split(&mut self, settings: &HashMap<String, String>) -> SplitResult {
        let mut result = SplitResult::new(self.config.criteria);
        let mut groups: HashMap<String, SplitGroup> = HashMap::new();

        for (key, value) in settings {
            let group_key = self.determine_group(key, value);

            if let Some(group_key) = group_key {
                let group = groups
                    .entry(group_key.clone())
                    .or_insert_with(|| SplitGroup::new(&group_key, &group_key));
                group.add(key.clone(), value.clone());
            } else {
                result.add_unmatched(key.clone());
            }
        }

        // Convert to result, respecting max_groups
        let mut sorted_groups: Vec<_> = groups.into_values().collect();
        sorted_groups.sort_by(|a, b| b.setting_count().cmp(&a.setting_count()));

        for group in sorted_groups.into_iter().take(self.config.max_groups) {
            result.add_group(group);
        }

        self.stats.record(
            self.config.criteria,
            result.group_count(),
            result.total_keys,
        );
        self.results.push(result.clone());
        result
    }

    /// Determine group for a key
    fn determine_group(&self, key: &str, value: &str) -> Option<String> {
        match self.config.criteria {
            SplitCriteria::ByCategory => {
                // Extract category from key (format: category.subcategory.setting)
                key.split('.').next().map(|s| s.to_string())
            }
            SplitCriteria::ByPrefix => {
                // Use first part before underscore or dot
                key.split(|c| c == '_' || c == '.')
                    .next()
                    .map(|s| s.to_string())
            }
            SplitCriteria::ByPattern => {
                // Group by first letter
                key.chars().next().map(|c| c.to_string())
            }
            SplitCriteria::ByValueType => {
                // Determine value type
                if value.parse::<i64>().is_ok() {
                    Some("integer".to_string())
                } else if value.parse::<f64>().is_ok() {
                    Some("float".to_string())
                } else if value == "true" || value == "false" {
                    Some("boolean".to_string())
                } else {
                    Some("string".to_string())
                }
            }
            SplitCriteria::BySize => {
                // Group by value length
                let len = value.len();
                if len < 10 {
                    Some("small".to_string())
                } else if len < 100 {
                    Some("medium".to_string())
                } else {
                    Some("large".to_string())
                }
            }
        }
    }

    /// Split into N groups evenly
    pub fn split_into_n(&mut self, settings: &HashMap<String, String>, n: usize) -> SplitResult {
        let mut result = SplitResult::new(self.config.criteria);
        let keys: Vec<_> = settings.keys().cloned().collect();
        let chunk_size = (keys.len() + n - 1) / n.max(1);

        for (i, chunk) in keys.chunks(chunk_size).enumerate() {
            let mut group = SplitGroup::new(format!("group_{}", i), format!("{}", i));
            for key in chunk {
                if let Some(value) = settings.get(key) {
                    group.add(key.clone(), value.clone());
                }
            }
            if !group.is_empty() {
                result.add_group(group);
            }
        }

        self.stats.record(
            self.config.criteria,
            result.group_count(),
            result.total_keys,
        );
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[SplitResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &SplitterStats {
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

/// Settings splitter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsSplitterRegistry {
    /// Splitters by ID
    splitters: HashMap<String, SettingsSplitter>,
}

impl SettingsSplitterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register splitter
    pub fn register(&mut self, id: impl Into<String>, splitter: SettingsSplitter) {
        self.splitters.insert(id.into(), splitter);
    }

    /// Unregister splitter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.splitters.remove(id).is_some()
    }

    /// Get splitter
    pub fn get(&self, id: &str) -> Option<&SettingsSplitter> {
        self.splitters.get(id)
    }

    /// Get splitter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSplitter> {
        self.splitters.get_mut(id)
    }

    /// Splitter count
    pub fn count(&self) -> usize {
        self.splitters.len()
    }
}

/// Format splitter registry
pub fn format_splitter_registry(registry: &SettingsSplitterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Splitter Registry:\n");
    output.push_str(&format!("  Splitters: {}\n", registry.count()));
    output
}

/// Check if query is about splitter
pub fn is_splitter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("splitter") || lower.contains("split settings") || lower.contains("divide settings")
}

/// Fun fact about splitter
pub fn splitter_fun_fact() -> &'static str {
    "Anna's settings splitters divide configs into manageable groups!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_criteria_display() {
        assert_eq!(format!("{}", SplitCriteria::ByCategory), "by_category");
        assert_eq!(format!("{}", SplitCriteria::ByPrefix), "by_prefix");
    }

    #[test]
    fn test_split_mode_display() {
        assert_eq!(format!("{}", SplitMode::Even), "even");
        assert_eq!(format!("{}", SplitMode::ByCount), "by_count");
    }

    #[test]
    fn test_config_new() {
        let c = SplitterConfig::new(SplitCriteria::ByPrefix);
        assert!(c.preserve_order);
    }

    #[test]
    fn test_config_builder() {
        let c = SplitterConfig::new(SplitCriteria::ByPattern)
            .mode(SplitMode::ByCount)
            .max_groups(5);
        assert_eq!(c.mode, SplitMode::ByCount);
        assert_eq!(c.max_groups, 5);
    }

    #[test]
    fn test_group_new() {
        let g = SplitGroup::new("test", "value");
        assert_eq!(g.name, "test");
        assert!(g.is_empty());
    }

    #[test]
    fn test_group_add() {
        let mut g = SplitGroup::new("test", "val");
        g.add("key1", "value1");
        g.add("key2", "value2");
        assert_eq!(g.setting_count(), 2);
    }

    #[test]
    fn test_result_new() {
        let r = SplitResult::new(SplitCriteria::ByCategory);
        assert_eq!(r.group_count(), 0);
    }

    #[test]
    fn test_result_add_group() {
        let mut r = SplitResult::new(SplitCriteria::ByCategory);
        let mut g = SplitGroup::new("test", "val");
        g.add("k", "v");
        r.add_group(g);
        assert_eq!(r.group_count(), 1);
        assert_eq!(r.total_keys, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = SplitterStats::default();
        s.record(SplitCriteria::ByCategory, 3, 15);
        assert_eq!(s.total_splits, 1);
        assert_eq!(s.total_groups, 3);
    }

    #[test]
    fn test_splitter_new() {
        let s = SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByCategory));
        assert_eq!(s.result_count(), 0);
    }

    #[test]
    fn test_splitter_split_by_category() {
        let mut s = SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByCategory));
        let mut settings = HashMap::new();
        settings.insert("ui.theme".to_string(), "dark".to_string());
        settings.insert("ui.font".to_string(), "mono".to_string());
        settings.insert("network.timeout".to_string(), "30".to_string());

        let r = s.split(&settings);
        assert_eq!(r.group_count(), 2);
    }

    #[test]
    fn test_splitter_split_into_n() {
        let mut s = SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByCategory));
        let mut settings = HashMap::new();
        for i in 0..10 {
            settings.insert(format!("key{}", i), format!("value{}", i));
        }

        let r = s.split_into_n(&settings, 3);
        assert!(r.group_count() <= 3);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsSplitterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsSplitterRegistry::new();
        r.register("s1", SettingsSplitter::new(SplitterConfig::new(SplitCriteria::ByPrefix)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_splitter_query() {
        assert!(is_splitter_query("settings splitter"));
        assert!(!is_splitter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = splitter_fun_fact();
        assert!(fact.contains("splitter"));
    }
}
