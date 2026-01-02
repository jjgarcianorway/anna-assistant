// v0.0.674: Settings Filter Helpers (Phase 250)
// Helper functions for settings filter

use super::registry::FilterRegistry;

/// Format filter registry
pub fn format_filter_registry(registry: &FilterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Filter Registry:\n");
    output.push_str(&format!("  Filters: {}\n", registry.count()));
    output
}

/// Check if query is about filter
pub fn is_filter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("filter settings") || lower.contains("settings filter") || lower.contains("exclude empty")
}

/// Fun fact about filter
pub fn filter_fun_fact() -> &'static str {
    "Anna's settings filter removes unwanted settings with powerful predicates!"
}
