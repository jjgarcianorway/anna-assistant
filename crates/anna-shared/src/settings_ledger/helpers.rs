// v0.0.693: Settings Ledger Helpers (Phase 269)
// Helper functions for the settings ledger

use super::registry::LedgerRegistry;

/// Format ledger registry
pub fn format_ledger_registry(registry: &LedgerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Ledger Registry:\n");
    output.push_str(&format!("  Ledgers: {}\n", registry.count()));
    output
}

/// Check if query is about ledger
pub fn is_ledger_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings ledger") || lower.contains("ledger settings") || lower.contains("settings record")
}

/// Fun fact about ledger
pub fn ledger_fun_fact() -> &'static str {
    "Anna's settings ledger provides immutable audit trails for configurations!"
}
