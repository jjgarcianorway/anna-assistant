// v0.0.707: Settings Journal (Phase 283)
// Journal statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::entry::JournalEntry;

/// Journal stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalStats {
    /// Total entries
    pub total_entries: usize,
    /// Total items
    pub total_items: usize,
    /// By mood
    pub by_mood: HashMap<String, usize>,
    /// Total tags
    pub total_tags: usize,
}

impl JournalStats {
    /// Update from journal
    pub fn update(&mut self, entries: &[JournalEntry]) {
        self.total_entries = entries.len();
        self.by_mood.clear();
        self.total_tags = 0;
        for entry in entries {
            *self.by_mood.entry(entry.mood.to_string()).or_insert(0) += 1;
            self.total_tags += entry.tags.len();
        }
    }

    /// Record item
    pub fn record_item(&mut self) {
        self.total_items += 1;
    }

    /// Avg tags per entry
    pub fn avg_tags(&self) -> f64 {
        if self.total_entries == 0 { 0.0 } else { self.total_tags as f64 / self.total_entries as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = JournalStats::default();
        let entries = vec![JournalEntry::new(1, "2025-12-15", "Title", "Content").tag("test")];
        s.update(&entries);
        assert_eq!(s.total_entries, 1);
        assert_eq!(s.total_tags, 1);
    }
}
