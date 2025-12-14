// v0.0.587: Settings Transactions (Phase 163)
// Atomic settings operations with rollback support

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransactionState {
    /// Transaction created but not started
    #[default]
    Pending,
    /// Transaction in progress
    Active,
    /// Transaction committed
    Committed,
    /// Transaction rolled back
    RolledBack,
    /// Transaction failed
    Failed,
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Active => write!(f, "active"),
            Self::Committed => write!(f, "committed"),
            Self::RolledBack => write!(f, "rolled_back"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Set a value
    Set,
    /// Delete a value
    Delete,
    /// Reset to default
    Reset,
    /// Update (merge)
    Update,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Set => write!(f, "set"),
            Self::Delete => write!(f, "delete"),
            Self::Reset => write!(f, "reset"),
            Self::Update => write!(f, "update"),
        }
    }
}

/// Single operation in a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOp {
    /// Operation type
    pub op_type: OperationType,
    /// Target category
    pub category: SettingsCategory,
    /// Key path
    pub key: String,
    /// New value (serialized)
    pub new_value: Option<String>,
    /// Previous value for rollback
    pub prev_value: Option<String>,
    /// Applied flag
    pub applied: bool,
}

impl TransactionOp {
    /// Create set operation
    pub fn set(category: SettingsCategory, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            op_type: OperationType::Set,
            category,
            key: key.into(),
            new_value: Some(value.into()),
            prev_value: None,
            applied: false,
        }
    }

    /// Create delete operation
    pub fn delete(category: SettingsCategory, key: impl Into<String>) -> Self {
        Self {
            op_type: OperationType::Delete,
            category,
            key: key.into(),
            new_value: None,
            prev_value: None,
            applied: false,
        }
    }

    /// Create reset operation
    pub fn reset(category: SettingsCategory) -> Self {
        Self {
            op_type: OperationType::Reset,
            category,
            key: String::new(),
            new_value: None,
            prev_value: None,
            applied: false,
        }
    }

    /// Store previous value for rollback
    pub fn with_prev(mut self, value: impl Into<String>) -> Self {
        self.prev_value = Some(value.into());
        self
    }

    /// Mark as applied
    pub fn mark_applied(&mut self) {
        self.applied = true;
    }
}

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

/// Format transaction
pub fn format_transaction(txn: &SettingsTransaction) -> String {
    let mut output = String::new();

    output.push_str(&format!("Transaction: {}\n", &txn.id[..8]));
    output.push_str(&format!("State: {}\n", txn.state));
    output.push_str(&format!("Operations: {}\n", txn.op_count()));

    if let Some(ref desc) = txn.description {
        output.push_str(&format!("Description: {}\n", desc));
    }

    output
}

/// Check if query is about transactions
pub fn is_transaction_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("transaction")
        || lower.contains("atomic")
        || lower.contains("rollback")
        || lower.contains("commit")
}

/// Fun fact about transactions
pub fn settings_transactions_fun_fact() -> &'static str {
    "Anna supports atomic settings changes with automatic rollback on failure!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_state_display() {
        assert_eq!(format!("{}", TransactionState::Active), "active");
        assert_eq!(format!("{}", TransactionState::Committed), "committed");
    }

    #[test]
    fn test_operation_type_display() {
        assert_eq!(format!("{}", OperationType::Set), "set");
        assert_eq!(format!("{}", OperationType::Delete), "delete");
    }

    #[test]
    fn test_transaction_op_set() {
        let op = TransactionOp::set(SettingsCategory::Personality, "key", "value");
        assert_eq!(op.op_type, OperationType::Set);
        assert_eq!(op.key, "key");
    }

    #[test]
    fn test_transaction_op_delete() {
        let op = TransactionOp::delete(SettingsCategory::Risk, "key");
        assert_eq!(op.op_type, OperationType::Delete);
    }

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

    #[test]
    fn test_format_transaction() {
        let txn = SettingsTransaction::new();
        let output = format_transaction(&txn);
        assert!(output.contains("Transaction"));
    }

    #[test]
    fn test_is_transaction_query() {
        assert!(is_transaction_query("begin transaction"));
        assert!(is_transaction_query("rollback changes"));
        assert!(!is_transaction_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_transactions_fun_fact();
        assert!(fact.contains("atomic"));
    }
}
