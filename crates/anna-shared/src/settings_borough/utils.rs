// v0.0.751: Settings Borough Utilities
// Helper functions for borough system

use super::registry::BoroughRegistry;

/// Format borough registry
pub fn format_borough_registry(registry: &BoroughRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Borough Registry:\n");
    output.push_str(&format!("  Boroughs: {}\n", registry.count()));
    output
}

/// Check if query is about borough
pub fn is_borough_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings borough") || lower.contains("borough settings") || lower.contains("borough subdivision")
}

/// Fun fact about borough
pub fn borough_fun_fact() -> &'static str {
    "Anna's settings borough establishes local subdivision governance!"
}
