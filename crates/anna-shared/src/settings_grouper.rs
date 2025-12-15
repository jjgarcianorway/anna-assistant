// v0.0.676: Settings Grouper (Phase 252)
// Group settings by various criteria

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Group by field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroupByField {
    /// Group by key prefix
    #[default]
    KeyPrefix,
    /// Group by key suffix
    KeySuffix,
    /// Group by value
    Value,
    /// Group by value type
    ValueType,
}

impl std::fmt::Display for GroupByField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyPrefix => write!(f, "key_prefix"),
            Self::KeySuffix => write!(f, "key_suffix"),
            Self::Value => write!(f, "value"),
            Self::ValueType => write!(f, "value_type"),
        }
    }
}

/// Value type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValueTypeClass {
    /// String type
    #[default]
    String,
    /// Number type
    Number,
    /// Boolean type
    Boolean,
    /// Empty type
    Empty,
}

impl std::fmt::Display for ValueTypeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::Empty => write!(f, "empty"),
        }
    }
}

/// Classify value type
pub fn classify_value(value: &str) -> ValueTypeClass {
    if value.is_empty() {
        ValueTypeClass::Empty
    } else if value == "true" || value == "false" {
        ValueTypeClass::Boolean
    } else if value.parse::<f64>().is_ok() {
        ValueTypeClass::Number
    } else {
        ValueTypeClass::String
    }
}

/// Grouper config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrouperConfig {
    /// Default group by field
    pub default_field: GroupByField,
    /// Prefix delimiter
    pub prefix_delimiter: String,
    /// Suffix delimiter
    pub suffix_delimiter: String,
    /// Min group size
    pub min_group_size: usize,
}

impl GrouperConfig {
    /// Create new config
    pub fn new(field: GroupByField) -> Self {
        Self {
            default_field: field,
            prefix_delimiter: ".".to_string(),
            suffix_delimiter: "_".to_string(),
            min_group_size: 1,
        }
    }

    /// Set prefix delimiter
    pub fn prefix_delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.prefix_delimiter = delimiter.into();
        self
    }

    /// Set suffix delimiter
    pub fn suffix_delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.suffix_delimiter = delimiter.into();
        self
    }

    /// Set min group size
    pub fn min_group_size(mut self, size: usize) -> Self {
        self.min_group_size = size;
        self
    }
}

impl Default for GrouperConfig {
    fn default() -> Self {
        Self::new(GroupByField::KeyPrefix)
    }
}

/// Settings group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsGroup {
    /// Group name
    pub name: String,
    /// Entries in group
    pub entries: Vec<(String, String)>,
    /// Count
    pub count: usize,
}

impl SettingsGroup {
    /// Create new group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
            count: 0,
        }
    }

    /// Add entry
    pub fn add(&mut self, key: String, value: String) {
        self.entries.push((key, value));
        self.count += 1;
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Group result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResult {
    /// Groups
    pub groups: HashMap<String, SettingsGroup>,
    /// Total entries
    pub total_entries: usize,
    /// Total groups
    pub total_groups: usize,
    /// Ungrouped count
    pub ungrouped: usize,
}

impl GroupResult {
    /// Create new result
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            total_entries: 0,
            total_groups: 0,
            ungrouped: 0,
        }
    }

    /// Add to group
    pub fn add_to_group(&mut self, group_name: &str, key: String, value: String) {
        let group = self.groups.entry(group_name.to_string())
            .or_insert_with(|| SettingsGroup::new(group_name));
        group.add(key, value);
        self.total_entries += 1;
    }

    /// Finalize
    pub fn finalize(&mut self) {
        self.total_groups = self.groups.len();
    }

    /// Get group names
    pub fn group_names(&self) -> Vec<&str> {
        self.groups.keys().map(|s| s.as_str()).collect()
    }

    /// Get group
    pub fn get_group(&self, name: &str) -> Option<&SettingsGroup> {
        self.groups.get(name)
    }
}

impl Default for GroupResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Grouper stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrouperStats {
    /// Total groupings
    pub total_groupings: usize,
    /// Total entries grouped
    pub total_entries: usize,
    /// Total groups created
    pub total_groups: usize,
    /// By field
    pub by_field: HashMap<String, usize>,
}

impl GrouperStats {
    /// Record grouping
    pub fn record(&mut self, result: &GroupResult, field: GroupByField) {
        self.total_groupings += 1;
        self.total_entries += result.total_entries;
        self.total_groups += result.total_groups;
        *self.by_field.entry(field.to_string()).or_insert(0) += 1;
    }

    /// Average groups per operation
    pub fn average_groups(&self) -> f64 {
        if self.total_groupings == 0 {
            0.0
        } else {
            self.total_groups as f64 / self.total_groupings as f64
        }
    }
}

/// Settings grouper
#[derive(Debug, Clone, Default)]
pub struct SettingsGrouper {
    /// Config
    config: GrouperConfig,
    /// Stats
    stats: GrouperStats,
}

impl SettingsGrouper {
    /// Create new grouper
    pub fn new(config: GrouperConfig) -> Self {
        Self {
            config,
            stats: GrouperStats::default(),
        }
    }

    /// Group by key prefix
    pub fn group_by_prefix(&mut self, settings: &HashMap<String, String>) -> GroupResult {
        let mut result = GroupResult::new();

        for (key, value) in settings {
            let prefix = key.split(&self.config.prefix_delimiter)
                .next()
                .unwrap_or(key);
            result.add_to_group(prefix, key.clone(), value.clone());
        }

        result.finalize();
        self.stats.record(&result, GroupByField::KeyPrefix);
        result
    }

    /// Group by key suffix
    pub fn group_by_suffix(&mut self, settings: &HashMap<String, String>) -> GroupResult {
        let mut result = GroupResult::new();

        for (key, value) in settings {
            let suffix = key.rsplit(&self.config.suffix_delimiter)
                .next()
                .unwrap_or(key);
            result.add_to_group(suffix, key.clone(), value.clone());
        }

        result.finalize();
        self.stats.record(&result, GroupByField::KeySuffix);
        result
    }

    /// Group by value type
    pub fn group_by_value_type(&mut self, settings: &HashMap<String, String>) -> GroupResult {
        let mut result = GroupResult::new();

        for (key, value) in settings {
            let type_class = classify_value(value);
            result.add_to_group(&type_class.to_string(), key.clone(), value.clone());
        }

        result.finalize();
        self.stats.record(&result, GroupByField::ValueType);
        result
    }

    /// Group by value
    pub fn group_by_value(&mut self, settings: &HashMap<String, String>) -> GroupResult {
        let mut result = GroupResult::new();

        for (key, value) in settings {
            result.add_to_group(value, key.clone(), value.clone());
        }

        result.finalize();
        self.stats.record(&result, GroupByField::Value);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &GrouperStats {
        &self.stats
    }
}

/// Grouper registry
#[derive(Debug, Clone, Default)]
pub struct GrouperRegistry {
    /// Groupers by ID
    groupers: HashMap<String, SettingsGrouper>,
}

impl GrouperRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register grouper
    pub fn register(&mut self, id: impl Into<String>, grouper: SettingsGrouper) {
        self.groupers.insert(id.into(), grouper);
    }

    /// Unregister grouper
    pub fn unregister(&mut self, id: &str) -> bool {
        self.groupers.remove(id).is_some()
    }

    /// Get grouper
    pub fn get(&self, id: &str) -> Option<&SettingsGrouper> {
        self.groupers.get(id)
    }

    /// Get grouper mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsGrouper> {
        self.groupers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.groupers.len()
    }
}

/// Format grouper registry
pub fn format_grouper_registry(registry: &GrouperRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Grouper Registry:\n");
    output.push_str(&format!("  Groupers: {}\n", registry.count()));
    output
}

/// Check if query is about grouper
pub fn is_grouper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("group settings") || lower.contains("settings grouper") || lower.contains("categorize settings")
}

/// Fun fact about grouper
pub fn grouper_fun_fact() -> &'static str {
    "Anna's settings grouper organizes your settings into logical categories!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_field_display() {
        assert_eq!(format!("{}", GroupByField::KeyPrefix), "key_prefix");
        assert_eq!(format!("{}", GroupByField::Value), "value");
    }

    #[test]
    fn test_value_type_class_display() {
        assert_eq!(format!("{}", ValueTypeClass::String), "string");
        assert_eq!(format!("{}", ValueTypeClass::Number), "number");
    }

    #[test]
    fn test_classify_value() {
        assert_eq!(classify_value("hello"), ValueTypeClass::String);
        assert_eq!(classify_value("123"), ValueTypeClass::Number);
        assert_eq!(classify_value("true"), ValueTypeClass::Boolean);
        assert_eq!(classify_value(""), ValueTypeClass::Empty);
    }

    #[test]
    fn test_config_new() {
        let c = GrouperConfig::new(GroupByField::KeyPrefix);
        assert_eq!(c.prefix_delimiter, ".");
    }

    #[test]
    fn test_config_builder() {
        let c = GrouperConfig::new(GroupByField::Value)
            .prefix_delimiter(":")
            .min_group_size(2);
        assert_eq!(c.prefix_delimiter, ":");
        assert_eq!(c.min_group_size, 2);
    }

    #[test]
    fn test_group_new() {
        let g = SettingsGroup::new("test");
        assert!(g.is_empty());
    }

    #[test]
    fn test_group_add() {
        let mut g = SettingsGroup::new("test");
        g.add("key".to_string(), "value".to_string());
        assert_eq!(g.count, 1);
        assert!(!g.is_empty());
    }

    #[test]
    fn test_result_new() {
        let r = GroupResult::new();
        assert_eq!(r.total_groups, 0);
    }

    #[test]
    fn test_result_add_to_group() {
        let mut r = GroupResult::new();
        r.add_to_group("g1", "k".to_string(), "v".to_string());
        r.finalize();
        assert_eq!(r.total_entries, 1);
        assert_eq!(r.total_groups, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = GrouperStats::default();
        let mut r = GroupResult::new();
        r.add_to_group("g1", "k".to_string(), "v".to_string());
        r.finalize();
        s.record(&r, GroupByField::KeyPrefix);
        assert_eq!(s.total_groupings, 1);
    }

    #[test]
    fn test_grouper_new() {
        let g = SettingsGrouper::new(GrouperConfig::default());
        assert_eq!(g.stats().total_groupings, 0);
    }

    #[test]
    fn test_grouper_by_prefix() {
        let mut g = SettingsGrouper::new(GrouperConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = g.group_by_prefix(&settings);
        assert_eq!(result.total_groups, 2);
        assert!(result.get_group("app").is_some());
        assert!(result.get_group("db").is_some());
    }

    #[test]
    fn test_grouper_by_value_type() {
        let mut g = SettingsGrouper::new(GrouperConfig::default());
        let mut settings = HashMap::new();
        settings.insert("count".to_string(), "42".to_string());
        settings.insert("name".to_string(), "test".to_string());
        settings.insert("enabled".to_string(), "true".to_string());

        let result = g.group_by_value_type(&settings);
        assert!(result.get_group("number").is_some());
        assert!(result.get_group("string").is_some());
        assert!(result.get_group("boolean").is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = GrouperRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GrouperRegistry::new();
        r.register("g1", SettingsGrouper::new(GrouperConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_grouper_query() {
        assert!(is_grouper_query("group settings"));
        assert!(!is_grouper_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = grouper_fun_fact();
        assert!(fact.contains("grouper"));
    }
}
