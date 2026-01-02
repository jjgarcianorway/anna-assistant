// v0.0.730: Settings Accord (Phase 306)
// Helper functions for settings accord

use super::registry::AccordRegistry;

/// Format accord registry
pub fn format_accord_registry(registry: &AccordRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Accord Registry:\n");
    output.push_str(&format!("  Accords: {}\n", registry.count()));
    output
}

/// Check if query is about accord
pub fn is_accord_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings accord") || lower.contains("accord settings") || lower.contains("formal agreement")
}

/// Fun fact about accord
pub fn accord_fun_fact() -> &'static str {
    "Anna's settings accord establishes formal governance agreements!"
}
