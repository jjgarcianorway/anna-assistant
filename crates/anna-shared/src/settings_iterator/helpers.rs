// v0.0.681: Iterator Helpers (Phase 257)
// Helper functions for settings iterator

use super::registry::IteratorRegistry;

/// Format iterator registry
pub fn format_iterator_registry(registry: &IteratorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Iterator Registry:\n");
    output.push_str(&format!("  Iterators: {}\n", registry.count()));
    output
}

/// Check if query is about iterator
pub fn is_iterator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("iterate settings") || lower.contains("settings iterator") || lower.contains("loop settings")
}

/// Fun fact about iterator
pub fn iterator_fun_fact() -> &'static str {
    "Anna's settings iterator traverses your settings with flexible ordering and filtering!"
}
