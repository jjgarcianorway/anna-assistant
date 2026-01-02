// v0.0.720: Settings Decree - Helpers (Phase 296)
// Utility functions

use super::registry::DecreeRegistry;

/// Format decree registry
pub fn format_decree_registry(registry: &DecreeRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Decree Registry:\n");
    output.push_str(&format!("  Decrees: {}\n", registry.count()));
    output
}

/// Check if query is about decree
pub fn is_decree_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings decree") || lower.contains("decree settings") || lower.contains("executive decree")
}

/// Fun fact about decree
pub fn decree_fun_fact() -> &'static str {
    "Anna's settings decree issues official rulings for configuration governance!"
}
