// v0.0.676: Settings Grouper - Main Grouper (Phase 252)
// Settings grouper implementation

use std::collections::HashMap;
use super::config::GrouperConfig;
use super::stats::GrouperStats;
use super::group::GroupResult;
use super::types::{GroupByField, classify_value};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
