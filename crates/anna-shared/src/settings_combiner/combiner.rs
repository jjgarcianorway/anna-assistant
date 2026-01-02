// v0.0.690: Settings Combiner Logic (Phase 266)
// Core settings merge implementation

use std::collections::HashMap;
use crate::settings_combiner::types::{
    CombinerConfig, CombinerStats, CombineResult, CombineConflict, CombineStrategy,
};

/// Settings combiner
#[derive(Debug, Clone, Default)]
pub struct SettingsCombiner {
    /// Config
    config: CombinerConfig,
    /// Stats
    stats: CombinerStats,
}

impl SettingsCombiner {
    /// Create new combiner
    pub fn new(config: CombinerConfig) -> Self {
        Self {
            config,
            stats: CombinerStats::default(),
        }
    }

    /// Merge two collections
    pub fn merge(&mut self, left: &HashMap<String, String>, right: &HashMap<String, String>) -> CombineResult {
        let mut merged = HashMap::new();
        let mut conflicts = Vec::new();
        let mut from_left = 0i32;
        let mut from_right = 0i32;

        // Add all left entries
        for (key, value) in left {
            if !self.config.preserve_empty && value.is_empty() {
                continue;
            }
            merged.insert(key.clone(), value.clone());
            from_left += 1;
        }

        // Process right entries
        for (key, value) in right {
            if !self.config.preserve_empty && value.is_empty() {
                continue;
            }

            if let Some(left_value) = left.get(key) {
                if left_value != value {
                    // Conflict
                    let conflict = CombineConflict::new(key.clone(), left_value.clone(), value.clone());

                    match self.config.strategy {
                        CombineStrategy::LeftWins => {
                            // Keep left (already in merged)
                        }
                        CombineStrategy::RightWins => {
                            merged.insert(key.clone(), value.clone());
                            from_right += 1;
                            from_left -= 1;
                        }
                        CombineStrategy::KeepBoth => {
                            let conflict_key = format!("{}{}", key, self.config.conflict_suffix);
                            merged.insert(conflict_key, value.clone());
                            from_right += 1;
                        }
                        CombineStrategy::ErrorOnConflict => {
                            conflicts.push(conflict);
                        }
                    }
                }
                // No conflict if values are equal
            } else {
                // New key from right
                merged.insert(key.clone(), value.clone());
                from_right += 1;
            }
        }

        let result = CombineResult::new(merged, conflicts, from_left.max(0) as usize, from_right.max(0) as usize);
        self.stats.record(&result, self.config.strategy);
        result
    }

    /// Merge multiple collections
    pub fn merge_all(&mut self, collections: &[HashMap<String, String>]) -> CombineResult {
        if collections.is_empty() {
            return CombineResult::default();
        }

        let mut result = collections[0].clone();
        for collection in collections.iter().skip(1) {
            let merge_result = self.merge(&result, collection);
            result = merge_result.merged;
        }

        CombineResult::new(result, Vec::new(), 0, 0)
    }

    /// Get stats
    pub fn stats(&self) -> &CombinerStats {
        &self.stats
    }
}
