// v0.0.693: Settings Ledger Entry
// Ledger entry and page structures

use serde::{Deserialize, Serialize};
use super::types::LedgerEntryType;

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
