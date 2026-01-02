// v0.0.713: Settings Notice Core (Phase 289)
// Main settings notice implementation

use super::config::NoticeConfig;
use super::entry::{NoticeEntry, NoticeMetadata};
use super::stats::NoticeStats;

/// Settings notice
#[derive(Debug, Clone, Default)]
pub struct SettingsNotice {
    /// Config
    config: NoticeConfig,
    /// Entries
    entries: Vec<NoticeEntry>,
    /// Metadata
    metadata: Vec<NoticeMetadata>,
    /// Stats
    stats: NoticeStats,
}

impl SettingsNotice {
    /// Create new notice system
    pub fn new(config: NoticeConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            metadata: Vec::new(),
            stats: NoticeStats::default(),
        }
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: NoticeEntry) -> bool {
        if self.entries.len() >= self.config.max_notices {
            return false;
        }
        self.entries.push(entry);
        self.update_stats();
        true
    }

    /// Get entry
    pub fn get_entry(&self, id: &str) -> Option<&NoticeEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get entry mut
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut NoticeEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Add metadata
    pub fn add_metadata(&mut self, meta: NoticeMetadata) {
        self.metadata.push(meta);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.entries, self.config.notice_type);
    }

    /// Get stats
    pub fn stats(&self) -> &NoticeStats {
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
    fn test_notice_new() {
        let n = SettingsNotice::new(NoticeConfig::default());
        assert_eq!(n.entry_count(), 0);
    }

    #[test]
    fn test_notice_add_entry() {
        let mut n = SettingsNotice::new(NoticeConfig::default());
        n.add_entry(NoticeEntry::new("e1", "Title", "Message"));
        assert_eq!(n.entry_count(), 1);
    }
}
