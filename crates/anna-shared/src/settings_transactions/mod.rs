// v0.0.587: Settings Transactions (Phase 163)
// Atomic settings operations with rollback support

mod types;
mod transaction;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{TransactionState, OperationType, TransactionOp};
pub use transaction::SettingsTransaction;
pub use manager::TransactionManager;
pub use utils::{format_transaction, is_transaction_query, settings_transactions_fun_fact};
