// v0.0.656: Settings Splitter Implementation (Phase 232)
// Main splitter logic

use std::collections::HashMap;

use super::types::{SplitCriteria, SplitGroup, SplitResult, SplitterConfig, SplitterStats};

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
