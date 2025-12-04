//! Installation ledger for tracking what Anna has installed/modified.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::AnnaError;
use crate::LEDGER_PATH;

/// Kind of ledger entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LedgerEntryKind {
    PackageInstalled,
    ModelPulled,
    FileCreated,
    DirectoryCreated,
    ConfigChanged,
    ServiceEnabled,
}

/// Single entry in the installation ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub kind: LedgerEntryKind,
    pub target: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Whether this is a base installation entry (survives reset)
    pub base: bool,
}

impl LedgerEntry {
    pub fn new(kind: LedgerEntryKind, target: String, base: bool) -> Self {
        Self {
            kind,
            target,
            timestamp: Utc::now(),
            metadata: None,
            base,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Installation ledger
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ledger {
    pub entries: Vec<LedgerEntry>,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

impl Ledger {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            entries: Vec::new(),
            created: now,
            last_modified: now,
        }
    }

    pub fn load() -> Result<Self, AnnaError> {
        let path = Path::new(LEDGER_PATH);
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let ledger: Ledger = serde_json::from_str(&content)?;
        Ok(ledger)
    }

    pub fn save(&self) -> Result<(), AnnaError> {
        let path = Path::new(LEDGER_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add(&mut self, entry: LedgerEntry) {
        self.entries.push(entry);
        self.last_modified = Utc::now();
    }

    pub fn reset_non_base(&mut self) {
        self.entries.retain(|e| e.base);
        self.last_modified = Utc::now();
    }

    pub fn summary(&self) -> LedgerSummary {
        let mut summary = LedgerSummary::default();
        for entry in &self.entries {
            match entry.kind {
                LedgerEntryKind::PackageInstalled => summary.packages += 1,
                LedgerEntryKind::ModelPulled => summary.models += 1,
                LedgerEntryKind::FileCreated => summary.files += 1,
                LedgerEntryKind::DirectoryCreated => summary.directories += 1,
                LedgerEntryKind::ConfigChanged => summary.configs += 1,
                LedgerEntryKind::ServiceEnabled => summary.services += 1,
            }
        }
        summary.total = self.entries.len();
        summary
    }
}

/// Summary of ledger contents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LedgerSummary {
    pub total: usize,
    pub packages: usize,
    pub models: usize,
    pub files: usize,
    pub directories: usize,
    pub configs: usize,
    pub services: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_new() {
        let ledger = Ledger::new();
        assert!(ledger.entries.is_empty());
    }

    #[test]
    fn test_ledger_add_entry() {
        let mut ledger = Ledger::new();
        ledger.add(LedgerEntry::new(
            LedgerEntryKind::PackageInstalled,
            "ollama".to_string(),
            true,
        ));
        assert_eq!(ledger.entries.len(), 1);
    }

    #[test]
    fn test_ledger_reset_non_base() {
        let mut ledger = Ledger::new();
        ledger.add(LedgerEntry::new(
            LedgerEntryKind::PackageInstalled,
            "ollama".to_string(),
            true,
        ));
        ledger.add(LedgerEntry::new(
            LedgerEntryKind::ModelPulled,
            "llama2".to_string(),
            false,
        ));
        ledger.reset_non_base();
        assert_eq!(ledger.entries.len(), 1);
    }
}
