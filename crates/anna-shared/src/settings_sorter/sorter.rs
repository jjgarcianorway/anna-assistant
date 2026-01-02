// v0.0.675: Settings Sorter Implementation (Phase 251)

use std::collections::HashMap;
use super::config::{SortCriteria, SorterConfig};
use super::types::{SortField, SortOrder, SortResult, SorterStats};

/// Settings sorter
#[derive(Debug, Clone, Default)]
pub struct SettingsSorter {
    /// Config
    config: SorterConfig,
    /// Stats
    stats: SorterStats,
}

impl SettingsSorter {
    /// Create new sorter
    pub fn new(config: SorterConfig) -> Self {
        Self {
            config,
            stats: SorterStats::default(),
        }
    }

    /// Sort by key
    pub fn sort_by_key(&mut self, settings: &HashMap<String, String>) -> SortResult {
        let criteria = SortCriteria::new(SortField::Key, self.config.default_order);
        self.sort_with_criteria(settings, &criteria)
    }

    /// Sort by value
    pub fn sort_by_value(&mut self, settings: &HashMap<String, String>) -> SortResult {
        let criteria = SortCriteria::new(SortField::Value, self.config.default_order);
        self.sort_with_criteria(settings, &criteria)
    }

    /// Sort with criteria
    pub fn sort_with_criteria(&mut self, settings: &HashMap<String, String>, criteria: &SortCriteria) -> SortResult {
        let mut entries: Vec<(String, String)> = settings.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if self.config.stable_sort {
            entries.sort_by(|a, b| {
                criteria.compare(
                    (a.0.as_str(), a.1.as_str()),
                    (b.0.as_str(), b.1.as_str()),
                    self.config.case_insensitive,
                )
            });
        } else {
            entries.sort_unstable_by(|a, b| {
                criteria.compare(
                    (a.0.as_str(), a.1.as_str()),
                    (b.0.as_str(), b.1.as_str()),
                    self.config.case_insensitive,
                )
            });
        }

        let result = SortResult::new(entries).with_criteria(vec![criteria.clone()]);
        self.stats.record(&result, criteria.order, criteria.field);
        result
    }

    /// Sort descending
    pub fn sort_descending(&mut self, settings: &HashMap<String, String>) -> SortResult {
        let criteria = SortCriteria::new(self.config.default_field, SortOrder::Descending);
        self.sort_with_criteria(settings, &criteria)
    }

    /// Get stats
    pub fn stats(&self) -> &SorterStats {
        &self.stats
    }
}
