// v0.0.693: Settings Ledger Types (Phase 269)
// Core types for the settings ledger system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::entry::LedgerEntry;

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
