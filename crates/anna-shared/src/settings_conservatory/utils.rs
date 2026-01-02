// v0.0.771: Conservatory Utilities
// Utility functions for conservatory operations

use super::registry::ConservatoryRegistry;

/// Format conservatory registry
pub fn format_conservatory_registry(registry: &ConservatoryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Conservatory Registry:\n");
    output.push_str(&format!("  Conservatories: {}\n", registry.count()));
    output
}

/// Check if query is about conservatory
pub fn is_conservatory_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings conservatory") || lower.contains("conservatory settings") || lower.contains("glass conservatory")
}

/// Fun fact about conservatory
pub fn conservatory_fun_fact() -> &'static str {
    "Anna's settings conservatory preserves configuration boundaries!"
}
