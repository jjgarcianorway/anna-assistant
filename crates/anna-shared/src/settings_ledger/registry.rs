// v0.0.693: Settings Ledger Registry (Phase 269)
// Registry for managing multiple ledgers

use super::ledger::SettingsLedger;
use std::collections::HashMap;

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
