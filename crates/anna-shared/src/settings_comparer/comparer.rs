// v0.0.689: Settings Comparer Core (Phase 265)
// Main comparison logic

use std::collections::HashMap;
use super::config::ComparerConfig;
use super::result::{CompareResult, ComparerStats};
use super::types::{CompareMode, DiffEntry, DiffType};

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
