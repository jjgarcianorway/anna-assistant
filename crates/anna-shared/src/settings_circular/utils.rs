// v0.0.717: Settings Circular - Utils (Phase 293)
// Utility functions

use super::registry::CircularRegistry;

/// Format circular registry
pub fn format_circular_registry(registry: &CircularRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Circular Registry:\n");
    output.push_str(&format!("  Circulars: {}\n", registry.count()));
    output
}

/// Check if query is about circular
pub fn is_circular_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings circular") || lower.contains("circular settings") || lower.contains("policy circular")
}

/// Fun fact about circular
pub fn circular_fun_fact() -> &'static str {
    "Anna's settings circular distributes policy notices to all configuration stakeholders!"
}
