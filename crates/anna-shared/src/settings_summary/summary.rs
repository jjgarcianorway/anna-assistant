// v0.0.711: Settings Summary Implementation (Phase 287)
// Main settings summary struct and methods

use super::types::{SummaryConfig, SummaryEntry, SummaryMetadata, SummaryStats};

/// Settings summary
#[derive(Debug, Clone, Default)]
pub struct SettingsSummary {
    /// Config
    config: SummaryConfig,
    /// Entries
    entries: Vec<SummaryEntry>,
    /// Metadata
    metadata: Vec<SummaryMetadata>,
    /// Stats
    stats: SummaryStats,
}

impl SettingsSummary {
    /// Create new summary
    pub fn new(config: SummaryConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            metadata: Vec::new(),
            stats: SummaryStats::default(),
        }
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: SummaryEntry) -> bool {
        if self.entries.len() >= self.config.max_entries {
            return false;
        }
        self.entries.push(entry);
        self.update_stats();
        true
    }

    /// Get entry
    pub fn get_entry(&self, id: &str) -> Option<&SummaryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get entry mut
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut SummaryEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Add metadata
    pub fn add_metadata(&mut self, meta: SummaryMetadata) {
        self.metadata.push(meta);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.entries, self.config.summary_type);
    }

    /// Get stats
    pub fn stats(&self) -> &SummaryStats {
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
    use crate::settings_summary::types::{SummaryConfig, SummaryEntry, SummaryMetadata};

    #[test]
    fn test_summary_new() {
        let s = SettingsSummary::new(SummaryConfig::default());
        assert_eq!(s.entry_count(), 0);
    }

    #[test]
    fn test_summary_add_entry() {
        let mut s = SettingsSummary::new(SummaryConfig::default());
        s.add_entry(SummaryEntry::new("e1", "key", "value"));
        assert_eq!(s.entry_count(), 1);
    }

    #[test]
    fn test_summary_add_metadata() {
        let mut s = SettingsSummary::new(SummaryConfig::default());
        s.add_metadata(SummaryMetadata::new("key", "value", "e1"));
        assert_eq!(s.metadata.len(), 1);
    }
}
