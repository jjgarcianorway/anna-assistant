// v0.0.587: Settings Transactions - Types
// Transaction state, operation types, and operation structures

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
}
