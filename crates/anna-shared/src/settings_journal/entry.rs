// v0.0.707: Settings Journal (Phase 283)
// Journal entries

use serde::{Deserialize, Serialize};
use super::enums::JournalMood;

/// Journal entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Entry ID
    pub id: usize,
    /// Date
    pub date: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Mood
    pub mood: JournalMood,
    /// Tags
    pub tags: Vec<String>,
}

impl JournalEntry {
    /// Create new entry
    pub fn new(id: usize, date: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id,
            date: date.into(),
            title: title.into(),
            content: content.into(),
            mood: JournalMood::Productive,
            tags: Vec::new(),
        }
    }

    /// Set mood
    pub fn mood(mut self, m: JournalMood) -> Self {
        self.mood = m;
        self
    }

    /// Add tag
    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tags.push(t.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_new() {
        let e = JournalEntry::new(1, "2025-12-15", "Title", "Content");
        assert_eq!(e.id, 1);
    }

    #[test]
    fn test_entry_builder() {
        let e = JournalEntry::new(1, "2025-12-15", "Title", "Content")
            .mood(JournalMood::Learning)
            .tag("config");
        assert_eq!(e.mood, JournalMood::Learning);
        assert_eq!(e.tags.len(), 1);
    }
}
