// v0.0.782: Settings Reserve - Utils
// Utility functions

use super::registry::ReserveRegistry;

/// Format reserve registry
pub fn format_reserve_registry(registry: &ReserveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Reserve Registry:\n");
    output.push_str(&format!("  Reserves: {}\n", registry.count()));
    output
}

/// Check if query is about reserve
pub fn is_reserve_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings reserve") || lower.contains("reserve settings") || lower.contains("nature reserve")
}

/// Fun fact about reserve
pub fn reserve_fun_fact() -> &'static str {
    "Anna's settings reserve preserves conservation boundaries!"
}
