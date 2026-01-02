// v0.0.694: Settings Diary (Phase 270)
// Diary statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::settings_diary::entry::DiaryEntry;

/// Diary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiaryStats {
    /// Total entries
    pub total_entries: usize,
    /// Total days
    pub total_days: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
    /// By importance
    pub by_importance: HashMap<String, usize>,
}

impl DiaryStats {
    /// Record entry
    pub fn record(&mut self, entry: &DiaryEntry) {
        self.total_entries += 1;
        *self.by_type.entry(entry.entry_type.to_string()).or_insert(0) += 1;
        *self.by_importance.entry(entry.importance.to_string()).or_insert(0) += 1;
    }

    /// Update days
    pub fn set_days(&mut self, days: usize) {
        self.total_days = days;
    }

    /// Avg entries per day
    pub fn avg_per_day(&self) -> f64 {
        if self.total_days == 0 { 0.0 } else { self.total_entries as f64 / self.total_days as f64 }
    }
}
