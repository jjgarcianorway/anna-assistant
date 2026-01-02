// v0.0.587: Settings Transactions - Transaction
// SettingsTransaction implementation

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

use super::types::{TransactionOp, TransactionState};

/// Settings transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsTransaction {
    /// Transaction ID
    pub id: String,
    /// State
    pub state: TransactionState,
    /// Operations
    pub operations: Vec<TransactionOp>,
    /// Created time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Started time
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completed time
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Description
    pub description: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl SettingsTransaction {
    /// Create new transaction
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: TransactionState::Pending,
            operations: Vec::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            description: None,
            error: None,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add operation
    pub fn add(&mut self, op: TransactionOp) {
        self.operations.push(op);
    }

    /// Add set operation
    pub fn set(&mut self, category: SettingsCategory, key: impl Into<String>, value: impl Into<String>) {
        self.add(TransactionOp::set(category, key, value));
    }

    /// Add delete operation
    pub fn delete(&mut self, category: SettingsCategory, key: impl Into<String>) {
        self.add(TransactionOp::delete(category, key));
    }

    /// Add reset operation
    pub fn reset(&mut self, category: SettingsCategory) {
        self.add(TransactionOp::reset(category));
    }

    /// Begin transaction
    pub fn begin(&mut self) -> bool {
        if self.state != TransactionState::Pending {
            return false;
        }
        self.state = TransactionState::Active;
        self.started_at = Some(chrono::Utc::now());
        true
    }

    /// Commit transaction
    pub fn commit(&mut self) -> bool {
        if self.state != TransactionState::Active {
            return false;
        }
        self.state = TransactionState::Committed;
        self.completed_at = Some(chrono::Utc::now());
        true
    }

    /// Rollback transaction
    pub fn rollback(&mut self) -> bool {
        if self.state != TransactionState::Active {
            return false;
        }
        self.state = TransactionState::RolledBack;
        self.completed_at = Some(chrono::Utc::now());
        true
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.state = TransactionState::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state, TransactionState::Committed | TransactionState::RolledBack | TransactionState::Failed)
    }

    /// Check if can commit
    pub fn can_commit(&self) -> bool {
        self.state == TransactionState::Active
    }

    /// Operation count
    pub fn op_count(&self) -> usize {
        self.operations.len()
    }

    /// Applied count
    pub fn applied_count(&self) -> usize {
        self.operations.iter().filter(|o| o.applied).count()
    }

    /// Get operations to rollback (reverse order)
    pub fn rollback_ops(&self) -> Vec<&TransactionOp> {
        self.operations.iter().filter(|o| o.applied).rev().collect()
    }
}

impl Default for SettingsTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let txn = SettingsTransaction::new();
        assert_eq!(txn.state, TransactionState::Pending);
        assert_eq!(txn.op_count(), 0);
    }

    #[test]
    fn test_transaction_lifecycle() {
        let mut txn = SettingsTransaction::new();
        assert!(txn.begin());
        assert_eq!(txn.state, TransactionState::Active);
        txn.set(SettingsCategory::Personality, "key", "value");
        assert!(txn.commit());
        assert_eq!(txn.state, TransactionState::Committed);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut txn = SettingsTransaction::new();
        txn.begin();
        txn.set(SettingsCategory::Risk, "level", "high");
        assert!(txn.rollback());
        assert_eq!(txn.state, TransactionState::RolledBack);
    }
}
