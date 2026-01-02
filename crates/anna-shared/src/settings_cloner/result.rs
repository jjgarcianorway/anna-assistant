// v0.0.657: Settings Cloner Results (Phase 233)
// Result types and statistics for cloning operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::CloneDepth;

/// Clone result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResult {
    /// Cloned settings
    pub cloned: HashMap<String, String>,
    /// Keys cloned
    pub keys_cloned: Vec<String>,
    /// Keys skipped
    pub keys_skipped: Vec<String>,
    /// Clone depth used
    pub depth: CloneDepth,
}

impl CloneResult {
    /// Create new result
    pub fn new(depth: CloneDepth) -> Self {
        Self {
            cloned: HashMap::new(),
            keys_cloned: Vec::new(),
            keys_skipped: Vec::new(),
            depth,
        }
    }

    /// Add cloned
    pub fn add_cloned(&mut self, original_key: String, new_key: String, value: String) {
        self.cloned.insert(new_key, value);
        self.keys_cloned.push(original_key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.keys_skipped.push(key);
    }

    /// Total cloned
    pub fn total_cloned(&self) -> usize {
        self.cloned.len()
    }

    /// Has skipped
    pub fn has_skipped(&self) -> bool {
        !self.keys_skipped.is_empty()
    }
}

/// Cloner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClonerStats {
    /// Total clones
    pub total_clones: usize,
    /// Total keys cloned
    pub total_keys_cloned: usize,
    /// Total keys skipped
    pub total_keys_skipped: usize,
    /// By depth
    pub by_depth: HashMap<String, usize>,
}

impl ClonerStats {
    /// Record clone
    pub fn record(&mut self, depth: CloneDepth, keys_cloned: usize, keys_skipped: usize) {
        self.total_clones += 1;
        self.total_keys_cloned += keys_cloned;
        self.total_keys_skipped += keys_skipped;
        *self.by_depth.entry(depth.to_string()).or_insert(0) += 1;
    }

    /// Clone success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_keys_cloned + self.total_keys_skipped;
        if total == 0 {
            0.0
        } else {
            self.total_keys_cloned as f64 / total as f64
        }
    }
}
