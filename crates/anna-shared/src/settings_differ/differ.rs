// v0.0.661: Settings Differ Core (Phase 237)
// Main differ implementation

use std::collections::HashMap;

use super::config::DifferConfig;
use super::entry::DiffEntry;
use super::result::DiffResult;
use super::stats::DifferStats;
use super::types::{DiffType, DiffMode};

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
