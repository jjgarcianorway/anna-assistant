// v0.0.587: Settings Transactions - Manager
// Transaction manager for handling active transactions and history

use super::transaction::SettingsTransaction;

/// Transaction manager
#[derive(Debug, Clone, Default)]
pub struct TransactionManager {
    /// Active transaction
    active: Option<SettingsTransaction>,
    /// Completed transactions
    history: Vec<SettingsTransaction>,
    /// Max history size
    max_history: usize,
}

impl TransactionManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Begin new transaction
    pub fn begin(&mut self) -> Option<&mut SettingsTransaction> {
        if self.active.is_some() {
            return None;
        }
        let mut txn = SettingsTransaction::new();
        txn.begin();
        self.active = Some(txn);
        self.active.as_mut()
    }

    /// Begin with description
    pub fn begin_with(&mut self, desc: impl Into<String>) -> Option<&mut SettingsTransaction> {
        if self.active.is_some() {
            return None;
        }
        let mut txn = SettingsTransaction::new().description(desc);
        txn.begin();
        self.active = Some(txn);
        self.active.as_mut()
    }

    /// Get active transaction
    pub fn active(&self) -> Option<&SettingsTransaction> {
        self.active.as_ref()
    }

    /// Get mutable active transaction
    pub fn active_mut(&mut self) -> Option<&mut SettingsTransaction> {
        self.active.as_mut()
    }

    /// Commit active transaction
    pub fn commit(&mut self) -> bool {
        if let Some(mut txn) = self.active.take() {
            if txn.commit() {
                self.add_to_history(txn);
                return true;
            }
            self.active = Some(txn);
        }
        false
    }

    /// Rollback active transaction
    pub fn rollback(&mut self) -> bool {
        if let Some(mut txn) = self.active.take() {
            if txn.rollback() {
                self.add_to_history(txn);
                return true;
            }
            self.active = Some(txn);
        }
        false
    }

    /// Add to history
    fn add_to_history(&mut self, txn: SettingsTransaction) {
        self.history.push(txn);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get history
    pub fn history(&self) -> &[SettingsTransaction] {
        &self.history
    }

    /// Has active transaction
    pub fn has_active(&self) -> bool {
        self.active.is_some()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_new() {
        let manager = TransactionManager::new();
        assert!(!manager.has_active());
    }

    #[test]
    fn test_manager_begin() {
        let mut manager = TransactionManager::new();
        assert!(manager.begin().is_some());
        assert!(manager.has_active());
    }

    #[test]
    fn test_manager_commit() {
        let mut manager = TransactionManager::new();
        manager.begin();
        assert!(manager.commit());
        assert!(!manager.has_active());
        assert_eq!(manager.history().len(), 1);
    }
}
