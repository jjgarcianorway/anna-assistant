// v0.0.693: Settings Ledger Implementation (Phase 269)
// Main ledger implementation

use super::types::{LedgerConfig, LedgerEntryType, LedgerStats};
use super::entry::{LedgerEntry, LedgerPage};

/// Settings ledger
#[derive(Debug, Clone, Default)]
pub struct SettingsLedger {
    /// Config
    config: LedgerConfig,
    /// Pages
    pages: Vec<LedgerPage>,
    /// Stats
    stats: LedgerStats,
    /// Next sequence
    next_seq: usize,
}

impl SettingsLedger {
    /// Create new ledger
    pub fn new(config: LedgerConfig) -> Self {
        let mut ledger = Self {
            config,
            pages: Vec::new(),
            stats: LedgerStats::default(),
            next_seq: 1,
        };
        ledger.pages.push(LedgerPage::new(1));
        ledger.stats.set_pages(1);
        ledger
    }

    /// Current page
    fn current_page_mut(&mut self) -> &mut LedgerPage {
        self.pages.last_mut().unwrap()
    }

    /// Record set
    pub fn record_set(&mut self, key: &str, value: &str) -> usize {
        let entry = LedgerEntry::new(self.next_seq, LedgerEntryType::Set, key)
            .value(value);
        let seq = self.next_seq;
        self.next_seq += 1;
        self.stats.record(&entry);
        self.current_page_mut().add(entry);
        seq
    }

    /// Record update
    pub fn record_update(&mut self, key: &str, old_value: &str, new_value: &str) -> usize {
        let entry = LedgerEntry::new(self.next_seq, LedgerEntryType::Update, key)
            .prev_value(old_value)
            .value(new_value);
        let seq = self.next_seq;
        self.next_seq += 1;
        self.stats.record(&entry);
        self.current_page_mut().add(entry);
        seq
    }

    /// Record delete
    pub fn record_delete(&mut self, key: &str, old_value: &str) -> usize {
        let entry = LedgerEntry::new(self.next_seq, LedgerEntryType::Delete, key)
            .prev_value(old_value);
        let seq = self.next_seq;
        self.next_seq += 1;
        self.stats.record(&entry);
        self.current_page_mut().add(entry);
        seq
    }

    /// Get entry by sequence
    pub fn get_entry(&self, seq: usize) -> Option<&LedgerEntry> {
        for page in &self.pages {
            for entry in &page.entries {
                if entry.sequence == seq {
                    return Some(entry);
                }
            }
        }
        None
    }

    /// Get stats
    pub fn stats(&self) -> &LedgerStats {
        &self.stats
    }

    /// Page count
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}
