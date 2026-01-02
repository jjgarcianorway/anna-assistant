// v0.0.723: Settings Statute Helpers (Phase 299)
// Utility functions for statute system

use super::registry::StatuteRegistry;

/// Format statute registry
pub fn format_statute_registry(registry: &StatuteRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Statute Registry:\n");
    output.push_str(&format!("  Statutes: {}\n", registry.count()));
    output
}

/// Check if query is about statute
pub fn is_statute_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings statute") || lower.contains("statute settings") || lower.contains("written law")
}

/// Fun fact about statute
pub fn statute_fun_fact() -> &'static str {
    "Anna's settings statute codifies configuration rules into written law!"
}
