// v0.0.694: Settings Diary (Phase 270)
// Daily page

use serde::{Deserialize, Serialize};
use crate::settings_diary::entry::DiaryEntry;

/// Daily page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPage {
    /// Date (YYYY-MM-DD)
    pub date: String,
    /// Entries
    pub entries: Vec<DiaryEntry>,
    /// Summary
    pub summary: Option<String>,
}

impl DailyPage {
    /// Create new page
    pub fn new(date: impl Into<String>) -> Self {
        Self {
            date: date.into(),
            entries: Vec::new(),
            summary: None,
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: DiaryEntry) {
        self.entries.push(entry);
    }

    /// Set summary
    pub fn summarize(&mut self, summary: impl Into<String>) {
        self.summary = Some(summary.into());
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Important entries
    pub fn important_entries(&self) -> Vec<&DiaryEntry> {
        self.entries.iter().filter(|e| e.is_important()).collect()
    }
}
