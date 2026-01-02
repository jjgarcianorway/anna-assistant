// v0.0.705: Settings Almanac (Phase 281)
// Almanac chapters and entries

use serde::{Deserialize, Serialize};

/// Almanac chapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacChapter {
    /// Chapter number
    pub number: usize,
    /// Title
    pub title: String,
    /// Period (e.g., "Week 1", "Q1")
    pub period: String,
    /// Entries
    pub entries: Vec<AlmanacEntry>,
}

impl AlmanacChapter {
    /// Create new chapter
    pub fn new(number: usize, title: impl Into<String>, period: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            period: period.into(),
            entries: Vec::new(),
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: AlmanacEntry) {
        self.entries.push(entry);
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Almanac entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Date
    pub date: String,
    /// Highlights
    pub highlight: bool,
}

impl AlmanacEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            date: date.into(),
            highlight: false,
        }
    }

    /// Set highlight
    pub fn highlight(mut self, h: bool) -> Self {
        self.highlight = h;
        self
    }
}
