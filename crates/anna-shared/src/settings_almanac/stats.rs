// v0.0.705: Settings Almanac (Phase 281)
// Almanac statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::settings_almanac::chapter::AlmanacChapter;

/// Almanac stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlmanacStats {
    /// Total chapters
    pub total_chapters: usize,
    /// Total entries
    pub total_entries: usize,
    /// Highlighted entries
    pub highlighted: usize,
    /// By period
    pub by_period: HashMap<String, usize>,
}

impl AlmanacStats {
    /// Update from almanac
    pub fn update(&mut self, chapters: &[AlmanacChapter]) {
        self.total_chapters = chapters.len();
        self.total_entries = chapters.iter().map(|c| c.entry_count()).sum();
        self.highlighted = chapters.iter()
            .flat_map(|c| &c.entries)
            .filter(|e| e.highlight)
            .count();
        self.by_period.clear();
        for ch in chapters {
            if !ch.period.is_empty() {
                *self.by_period.entry(ch.period.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Highlight rate
    pub fn highlight_rate(&self) -> f64 {
        if self.total_entries == 0 { 0.0 } else { self.highlighted as f64 / self.total_entries as f64 * 100.0 }
    }
}
