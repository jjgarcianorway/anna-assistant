// v0.0.689: Settings Comparer Result (Phase 265)
// Result and stats types for comparison

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{DiffEntry, DiffType, CompareMode};

/// Compare result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    /// Diff entries
    pub entries: Vec<DiffEntry>,
    /// Total left
    pub total_left: usize,
    /// Total right
    pub total_right: usize,
    /// Added count
    pub added: usize,
    /// Removed count
    pub removed: usize,
    /// Changed count
    pub changed: usize,
    /// Unchanged count
    pub unchanged: usize,
}

impl CompareResult {
    /// Create new result
    pub fn new(entries: Vec<DiffEntry>, left: usize, right: usize) -> Self {
        let added = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Added)).count();
        let removed = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Removed)).count();
        let changed = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Changed)).count();
        let unchanged = entries.iter().filter(|e| matches!(e.diff_type, DiffType::Unchanged)).count();

        Self {
            entries,
            total_left: left,
            total_right: right,
            added,
            removed,
            changed,
            unchanged,
        }
    }

    /// Has changes
    pub fn has_changes(&self) -> bool {
        self.added > 0 || self.removed > 0 || self.changed > 0
    }

    /// Are identical
    pub fn are_identical(&self) -> bool {
        !self.has_changes()
    }

    /// Filter by type
    pub fn filter_by_type(&self, diff_type: DiffType) -> Vec<&DiffEntry> {
        self.entries.iter().filter(|e| e.diff_type == diff_type).collect()
    }

    /// Change summary
    pub fn summary(&self) -> String {
        format!("+{} -{} ~{}", self.added, self.removed, self.changed)
    }
}

impl Default for CompareResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0)
    }
}

/// Comparer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparerStats {
    /// Total comparisons
    pub total_comparisons: usize,
    /// Total entries compared
    pub total_entries: usize,
    /// Total changes found
    pub total_changes: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ComparerStats {
    /// Record comparison
    pub fn record(&mut self, result: &CompareResult, mode: CompareMode) {
        self.total_comparisons += 1;
        self.total_entries += result.total_left + result.total_right;
        self.total_changes += result.added + result.removed + result.changed;
        *self.by_mode.entry(mode.to_string()).or_insert(0) += 1;
    }

    /// Avg changes per comparison
    pub fn avg_changes(&self) -> f64 {
        if self.total_comparisons == 0 {
            0.0
        } else {
            self.total_changes as f64 / self.total_comparisons as f64
        }
    }
}
