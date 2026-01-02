// v0.0.587: Settings Transactions - Utilities
// Helper functions for formatting and queries

use super::transaction::SettingsTransaction;

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
    use crate::settings_transactions::SettingsTransaction;

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
