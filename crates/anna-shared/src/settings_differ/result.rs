// v0.0.661: Settings Differ Result (Phase 237)
// Diff result representation

use serde::{Deserialize, Serialize};

use super::entry::DiffEntry;
use super::types::DiffType;

/// Diff result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// All diff entries
    pub entries: Vec<DiffEntry>,
    /// Count by type
    pub added_count: usize,
    /// Removed count
    pub removed_count: usize,
    /// Modified count
    pub modified_count: usize,
    /// Unchanged count
    pub unchanged_count: usize,
}

impl DiffResult {
    /// Create new result
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            added_count: 0,
            removed_count: 0,
            modified_count: 0,
            unchanged_count: 0,
        }
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: DiffEntry) {
        match entry.diff_type {
            DiffType::Added => self.added_count += 1,
            DiffType::Removed => self.removed_count += 1,
            DiffType::Modified => self.modified_count += 1,
            DiffType::Unchanged => self.unchanged_count += 1,
        }
        self.entries.push(entry);
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.added_count + self.removed_count + self.modified_count
    }

    /// Has changes
    pub fn has_changes(&self) -> bool {
        self.total_changes() > 0
    }

    /// Get entries by type
    pub fn get_by_type(&self, diff_type: DiffType) -> Vec<&DiffEntry> {
        self.entries.iter().filter(|e| e.diff_type == diff_type).collect()
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
}
