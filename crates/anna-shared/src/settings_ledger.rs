// v0.0.693: Settings Ledger (Phase 269)
// Ledger for immutable settings records

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ledger entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LedgerEntryType {
    /// Set value
    #[default]
    Set,
    /// Update value
    Update,
    /// Delete value
    Delete,
    /// Import batch
    Import,
}

impl std::fmt::Display for LedgerEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Set => write!(f, "set"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::Import => write!(f, "import"),
        }
    }
}

/// Ledger status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LedgerStatus {
    /// Active
    #[default]
    Active,
    /// Archived
    Archived,
    /// Sealed
    Sealed,
    /// Pending
    Pending,
}

impl std::fmt::Display for LedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::Sealed => write!(f, "sealed"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

/// Ledger config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerConfig {
    /// Name
    pub name: String,
    /// Status
    pub status: LedgerStatus,
    /// Max entries
    pub max_entries: usize,
    /// Immutable
    pub immutable: bool,
}

impl LedgerConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: LedgerStatus::Active,
            max_entries: 10000,
            immutable: true,
        }
    }

    /// Set max entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set immutable
    pub fn immutable(mut self, immutable: bool) -> Self {
        self.immutable = immutable;
        self
    }
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Ledger entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Sequence number
    pub sequence: usize,
    /// Entry type
    pub entry_type: LedgerEntryType,
    /// Key
    pub key: String,
    /// Value
    pub value: Option<String>,
    /// Previous value
    pub prev_value: Option<String>,
    /// Hash
    pub hash: String,
}

impl LedgerEntry {
    /// Create new entry
    pub fn new(seq: usize, entry_type: LedgerEntryType, key: impl Into<String>) -> Self {
        let key_str = key.into();
        let hash = format!("{:x}", seq.wrapping_mul(31).wrapping_add(key_str.len()));
        Self {
            sequence: seq,
            entry_type,
            key: key_str,
            value: None,
            prev_value: None,
            hash,
        }
    }

    /// Set value
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = Some(val.into());
        self
    }

    /// Set previous value
    pub fn prev_value(mut self, val: impl Into<String>) -> Self {
        self.prev_value = Some(val.into());
        self
    }

    /// Is modification
    pub fn is_modification(&self) -> bool {
        !matches!(self.entry_type, LedgerEntryType::Import)
    }
}

/// Ledger page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerPage {
    /// Page number
    pub page_number: usize,
    /// Entries
    pub entries: Vec<LedgerEntry>,
    /// Is sealed
    pub sealed: bool,
}

impl LedgerPage {
    /// Create new page
    pub fn new(page_number: usize) -> Self {
        Self {
            page_number,
            entries: Vec::new(),
            sealed: false,
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: LedgerEntry) -> bool {
        if self.sealed {
            return false;
        }
        self.entries.push(entry);
        true
    }

    /// Seal page
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

/// Ledger stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LedgerStats {
    /// Total entries
    pub total_entries: usize,
    /// Total pages
    pub total_pages: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl LedgerStats {
    /// Record entry
    pub fn record(&mut self, entry: &LedgerEntry) {
        self.total_entries += 1;
        *self.by_type.entry(entry.entry_type.to_string()).or_insert(0) += 1;
    }

    /// Update pages
    pub fn set_pages(&mut self, count: usize) {
        self.total_pages = count;
    }
}

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

/// Ledger registry
#[derive(Debug, Clone, Default)]
pub struct LedgerRegistry {
    /// Ledgers by ID
    ledgers: HashMap<String, SettingsLedger>,
}

impl LedgerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register ledger
    pub fn register(&mut self, id: impl Into<String>, ledger: SettingsLedger) {
        self.ledgers.insert(id.into(), ledger);
    }

    /// Unregister ledger
    pub fn unregister(&mut self, id: &str) -> bool {
        self.ledgers.remove(id).is_some()
    }

    /// Get ledger
    pub fn get(&self, id: &str) -> Option<&SettingsLedger> {
        self.ledgers.get(id)
    }

    /// Get ledger mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsLedger> {
        self.ledgers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.ledgers.len()
    }
}

/// Format ledger registry
pub fn format_ledger_registry(registry: &LedgerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Ledger Registry:\n");
    output.push_str(&format!("  Ledgers: {}\n", registry.count()));
    output
}

/// Check if query is about ledger
pub fn is_ledger_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings ledger") || lower.contains("ledger settings") || lower.contains("settings record")
}

/// Fun fact about ledger
pub fn ledger_fun_fact() -> &'static str {
    "Anna's settings ledger provides immutable audit trails for configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_display() {
        assert_eq!(format!("{}", LedgerEntryType::Set), "set");
        assert_eq!(format!("{}", LedgerEntryType::Update), "update");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", LedgerStatus::Active), "active");
        assert_eq!(format!("{}", LedgerStatus::Sealed), "sealed");
    }

    #[test]
    fn test_config_new() {
        let c = LedgerConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = LedgerConfig::new("test")
            .max_entries(500)
            .immutable(false);
        assert_eq!(c.max_entries, 500);
        assert!(!c.immutable);
    }

    #[test]
    fn test_entry_new() {
        let e = LedgerEntry::new(1, LedgerEntryType::Set, "key");
        assert!(e.is_modification());
    }

    #[test]
    fn test_entry_values() {
        let e = LedgerEntry::new(1, LedgerEntryType::Update, "key")
            .prev_value("old")
            .value("new");
        assert_eq!(e.prev_value, Some("old".to_string()));
        assert_eq!(e.value, Some("new".to_string()));
    }

    #[test]
    fn test_page_new() {
        let p = LedgerPage::new(1);
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_page_add() {
        let mut p = LedgerPage::new(1);
        p.add(LedgerEntry::new(1, LedgerEntryType::Set, "key"));
        assert_eq!(p.count(), 1);
    }

    #[test]
    fn test_page_seal() {
        let mut p = LedgerPage::new(1);
        p.seal();
        let added = p.add(LedgerEntry::new(1, LedgerEntryType::Set, "key"));
        assert!(!added);
    }

    #[test]
    fn test_stats_record() {
        let mut s = LedgerStats::default();
        s.record(&LedgerEntry::new(1, LedgerEntryType::Set, "key"));
        assert_eq!(s.total_entries, 1);
    }

    #[test]
    fn test_ledger_new() {
        let l = SettingsLedger::new(LedgerConfig::default());
        assert_eq!(l.page_count(), 1);
    }

    #[test]
    fn test_ledger_record_set() {
        let mut l = SettingsLedger::new(LedgerConfig::default());
        let seq = l.record_set("key", "value");
        assert_eq!(seq, 1);
        assert_eq!(l.stats().total_entries, 1);
    }

    #[test]
    fn test_ledger_record_update() {
        let mut l = SettingsLedger::new(LedgerConfig::default());
        let seq = l.record_update("key", "old", "new");
        assert_eq!(seq, 1);
    }

    #[test]
    fn test_ledger_get_entry() {
        let mut l = SettingsLedger::new(LedgerConfig::default());
        l.record_set("key", "value");
        let entry = l.get_entry(1);
        assert!(entry.is_some());
    }

    #[test]
    fn test_registry_new() {
        let r = LedgerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = LedgerRegistry::new();
        r.register("l1", SettingsLedger::new(LedgerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_ledger_query() {
        assert!(is_ledger_query("settings ledger"));
        assert!(!is_ledger_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = ledger_fun_fact();
        assert!(fact.contains("ledger"));
    }
}
