//! Multi-file change transactions for atomic config modifications.
//!
//! v0.0.98: Allows grouping multiple changes into a single transaction.
//! If any change fails, all changes are rolled back.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::change::{apply_change, rollback, ChangePlan, ChangeResult};

/// A transaction containing multiple change plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTransaction {
    /// Unique transaction ID
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// List of changes to apply atomically
    pub changes: Vec<ChangePlan>,
    /// Overall risk level (highest of all changes)
    pub risk: crate::change::ChangeRisk,
}

impl ChangeTransaction {
    /// Create a new transaction
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: generate_tx_id(),
            description: description.into(),
            changes: vec![],
            risk: crate::change::ChangeRisk::Low,
        }
    }

    /// Add a change to the transaction
    pub fn add(&mut self, plan: ChangePlan) {
        // Update risk level to highest
        if plan.risk as u8 > self.risk as u8 {
            self.risk = plan.risk;
        }
        self.changes.push(plan);
    }

    /// Check if transaction is empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Check if all changes are no-ops
    pub fn is_noop(&self) -> bool {
        self.changes.iter().all(|c| c.is_noop)
    }

    /// Get number of actual changes (non-noop)
    pub fn change_count(&self) -> usize {
        self.changes.iter().filter(|c| !c.is_noop).count()
    }

    /// Get summary of changes
    pub fn summary(&self) -> String {
        if self.is_noop() {
            format!("{}: No changes needed", self.description)
        } else {
            let count = self.change_count();
            format!(
                "{}: {} file{} to modify",
                self.description,
                count,
                if count == 1 { "" } else { "s" }
            )
        }
    }

    /// List all target files
    pub fn target_files(&self) -> Vec<&PathBuf> {
        self.changes.iter().map(|c| &c.target_path).collect()
    }
}

/// Result of applying a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// Transaction ID
    pub tx_id: String,
    /// Whether all changes were applied successfully
    pub success: bool,
    /// Whether it was a complete no-op
    pub was_noop: bool,
    /// Number of changes applied
    pub changes_applied: usize,
    /// Number of changes rolled back (if failure)
    pub changes_rolled_back: usize,
    /// Individual change results
    pub results: Vec<ChangeResult>,
    /// Error message if failed
    pub error: Option<String>,
}

impl TransactionResult {
    fn success(tx_id: String, results: Vec<ChangeResult>) -> Self {
        let changes_applied = results.iter().filter(|r| r.applied).count();
        let was_noop = results.iter().all(|r| r.was_noop);
        Self {
            tx_id,
            success: true,
            was_noop,
            changes_applied,
            changes_rolled_back: 0,
            results,
            error: None,
        }
    }

    fn failed(tx_id: String, results: Vec<ChangeResult>, rolled_back: usize, error: String) -> Self {
        Self {
            tx_id,
            success: false,
            was_noop: false,
            changes_applied: 0,
            changes_rolled_back: rolled_back,
            results,
            error: Some(error),
        }
    }
}

/// Apply a transaction atomically
/// If any change fails, all previously applied changes are rolled back
pub fn apply_transaction(tx: &ChangeTransaction) -> TransactionResult {
    if tx.is_empty() {
        return TransactionResult::success(tx.id.clone(), vec![]);
    }

    if tx.is_noop() {
        let results: Vec<ChangeResult> = tx.changes.iter()
            .map(|_| ChangeResult::noop())
            .collect();
        return TransactionResult::success(tx.id.clone(), results);
    }

    let mut results: Vec<ChangeResult> = Vec::new();
    let mut applied_plans: Vec<&ChangePlan> = Vec::new();

    // Apply each change
    for plan in &tx.changes {
        let result = apply_change(plan);

        if result.applied {
            applied_plans.push(plan);
        } else if result.error.is_some() {
            // Failure - rollback all applied changes
            let error = result.error.clone().unwrap_or_default();
            results.push(result);

            let rolled_back = rollback_applied(&applied_plans);

            return TransactionResult::failed(
                tx.id.clone(),
                results,
                rolled_back,
                format!("Transaction failed: {}", error),
            );
        }

        results.push(result);
    }

    TransactionResult::success(tx.id.clone(), results)
}

/// Rollback all applied changes (in reverse order)
fn rollback_applied(plans: &[&ChangePlan]) -> usize {
    let mut rolled_back = 0;

    for plan in plans.iter().rev() {
        let result = rollback(plan);
        if result.applied || result.was_noop {
            rolled_back += 1;
        }
    }

    rolled_back
}

/// Generate a unique transaction ID
fn generate_tx_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("tx-{:08x}", (duration.as_nanos() % u32::MAX as u128) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_transaction() {
        let tx = ChangeTransaction::new("test");
        assert!(tx.is_empty());
        assert!(tx.is_noop());
        assert_eq!(tx.change_count(), 0);
    }

    #[test]
    fn test_transaction_summary() {
        let tx = ChangeTransaction::new("Configure vim");
        assert!(tx.summary().contains("No changes needed"));
    }
}
