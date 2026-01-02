// v0.0.707: Settings Journal (Phase 283)
// Main journal implementation

use super::config::JournalConfig;
use super::entry::JournalEntry;
use super::item::JournalItem;
use super::stats::JournalStats;

/// Settings journal
#[derive(Debug, Clone, Default)]
pub struct SettingsJournal {
    /// Config
    config: JournalConfig,
    /// Entries
    entries: Vec<JournalEntry>,
    /// Items
    items: Vec<JournalItem>,
    /// Stats
    stats: JournalStats,
    /// Next ID
    next_id: usize,
}

impl SettingsJournal {
    /// Create new journal
    pub fn new(config: JournalConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            items: Vec::new(),
            stats: JournalStats::default(),
            next_id: 1,
        }
    }

    /// Write entry
    pub fn write(&mut self, date: &str, title: &str, content: &str) -> usize {
        if self.entries.len() >= self.config.max_entries {
            return 0;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(JournalEntry::new(id, date, title, content));
        self.update_stats();
        id
    }

    /// Get entry
    pub fn get_entry(&self, id: usize) -> Option<&JournalEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Add item
    pub fn add_item(&mut self, item: JournalItem) {
        self.items.push(item);
        self.stats.record_item();
    }

    /// Get items for entry
    pub fn get_items(&self, entry_id: usize) -> Vec<&JournalItem> {
        self.items.iter().filter(|i| i.entry_id == entry_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.entries);
    }

    /// Get stats
    pub fn stats(&self) -> &JournalStats {
        &self.stats
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_new() {
        let j = SettingsJournal::new(JournalConfig::default());
        assert_eq!(j.entry_count(), 0);
    }

    #[test]
    fn test_journal_write() {
        let mut j = SettingsJournal::new(JournalConfig::default());
        let id = j.write("2025-12-15", "Title", "Content");
        assert_eq!(id, 1);
        assert_eq!(j.entry_count(), 1);
    }

    #[test]
    fn test_journal_add_item() {
        let mut j = SettingsJournal::new(JournalConfig::default());
        j.add_item(JournalItem::new("key", "value", 1));
        assert_eq!(j.stats().total_items, 1);
    }
}
