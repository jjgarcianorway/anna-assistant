// v0.0.669: Settings Indexer Helpers (Phase 245)
// Helper functions for settings indexer

use super::registry::IndexerRegistry;

/// Format indexer registry
pub fn format_indexer_registry(registry: &IndexerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Indexer Registry:\n");
    output.push_str(&format!("  Indexers: {}\n", registry.count()));
    output
}

/// Check if query is about indexer
pub fn is_indexer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("index") || lower.contains("settings index") || lower.contains("search settings")
}

/// Fun fact about indexer
pub fn indexer_fun_fact() -> &'static str {
    "Anna's settings indexer enables fast lookup and full-text search!"
}
