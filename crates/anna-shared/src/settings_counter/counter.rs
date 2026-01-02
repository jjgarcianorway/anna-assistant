// v0.0.686: Settings Counter Implementation (Phase 262)
// Main counter implementation

use std::collections::HashMap;
use super::types::{CountType, CounterConfig, CounterStats, CountEntry, CountResult, ValueType};

/// Settings counter
#[derive(Debug, Clone, Default)]
pub struct SettingsCounter {
    /// Config
    config: CounterConfig,
    /// Stats
    stats: CounterStats,
}

impl SettingsCounter {
    /// Create new counter
    pub fn new(config: CounterConfig) -> Self {
        Self {
            config,
            stats: CounterStats::default(),
        }
    }

    /// Detect value type
    fn detect_type(&self, value: &str) -> ValueType {
        if value.is_empty() {
            ValueType::Empty
        } else if value == "true" || value == "false" {
            ValueType::Boolean
        } else if value.parse::<f64>().is_ok() {
            ValueType::Numeric
        } else {
            ValueType::String
        }
    }

    /// Extract prefix
    fn extract_prefix(&self, key: &str) -> String {
        key.split(self.config.prefix_separator)
            .next()
            .unwrap_or(key)
            .to_string()
    }

    /// Count all
    pub fn count_all(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let total = if self.config.include_empty {
            settings.len()
        } else {
            settings.values().filter(|v| !v.is_empty()).count()
        };

        let entries = vec![CountEntry::new("total", total, total)];
        let result = CountResult::new(total, entries, CountType::All);
        self.stats.record(&result);
        result
    }

    /// Count by prefix
    pub fn count_by_prefix(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for key in settings.keys() {
            let prefix = self.extract_prefix(key);
            *counts.entry(prefix).or_insert(0) += 1;
        }

        let total = settings.len();
        let mut entries: Vec<CountEntry> = counts
            .into_iter()
            .filter(|(_, c)| *c >= self.config.group_threshold)
            .map(|(label, count)| CountEntry::new(label, count, total))
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count));

        let result = CountResult::new(total, entries, CountType::ByPrefix);
        self.stats.record(&result);
        result
    }

    /// Count by value type
    pub fn count_by_value_type(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let mut counts: HashMap<ValueType, usize> = HashMap::new();

        for value in settings.values() {
            if !self.config.include_empty && value.is_empty() {
                continue;
            }
            let vtype = self.detect_type(value);
            *counts.entry(vtype).or_insert(0) += 1;
        }

        let total = settings.len();
        let mut entries: Vec<CountEntry> = counts
            .into_iter()
            .map(|(vtype, count)| CountEntry::new(vtype.to_string(), count, total))
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count));

        let result = CountResult::new(total, entries, CountType::ByValueType);
        self.stats.record(&result);
        result
    }

    /// Count by value length
    pub fn count_by_length(&mut self, settings: &HashMap<String, String>) -> CountResult {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for value in settings.values() {
            let len = value.len();
            let bucket = if len == 0 {
                "empty".to_string()
            } else if len <= 10 {
                "short (1-10)".to_string()
            } else if len <= 50 {
                "medium (11-50)".to_string()
            } else if len <= 200 {
                "long (51-200)".to_string()
            } else {
                "very_long (200+)".to_string()
            };
            *counts.entry(bucket).or_insert(0) += 1;
        }

        let total = settings.len();
        let mut entries: Vec<CountEntry> = counts
            .into_iter()
            .map(|(label, count)| CountEntry::new(label, count, total))
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count));

        let result = CountResult::new(total, entries, CountType::ByLength);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &CounterStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_new() {
        let c = SettingsCounter::new(CounterConfig::default());
        assert_eq!(c.stats().total_counts, 0);
    }

    #[test]
    fn test_counter_count_all() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());

        let result = c.count_all(&settings);
        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_counter_by_prefix() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = c.count_by_prefix(&settings);
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_counter_by_value_type() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("name".to_string(), "test".to_string());
        settings.insert("count".to_string(), "42".to_string());
        settings.insert("enabled".to_string(), "true".to_string());

        let result = c.count_by_value_type(&settings);
        assert!(result.entries.len() > 0);
    }

    #[test]
    fn test_counter_by_length() {
        let mut c = SettingsCounter::new(CounterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("short".to_string(), "abc".to_string());
        settings.insert("medium".to_string(), "abcdefghijklmnopqrstuvwxyz".to_string());

        let result = c.count_by_length(&settings);
        assert!(result.entries.len() > 0);
    }
}
