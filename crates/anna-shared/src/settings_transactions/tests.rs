// v0.0.587: Settings Transactions - Integration Tests
// Tests for the complete transaction system

#[cfg(test)]
mod integration_tests {
    use crate::settings_transactions::*;
    use crate::unified_settings::SettingsCategory;

    // Note: Individual unit tests are in each module file:
    // - types.rs: TransactionState, OperationType, TransactionOp tests
    // - transaction.rs: SettingsTransaction tests
    // - manager.rs: TransactionManager tests
    // - utils.rs: utility function tests

    // This file contains integration tests that test multiple components together

    #[test]
    fn test_full_transaction_workflow() {
        let mut manager = TransactionManager::new();

        // Begin transaction
        let txn = manager.begin();
        assert!(txn.is_some());

        // Add operations
        if let Some(t) = manager.active_mut() {
            t.set(SettingsCategory::Personality, "name", "Anna");
            t.set(SettingsCategory::Risk, "level", "low");
            assert_eq!(t.op_count(), 2);
        }

        // Commit
        assert!(manager.commit());
        assert!(!manager.has_active());
        assert_eq!(manager.history().len(), 1);
    }
}
