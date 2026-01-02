// v0.0.705: Settings Almanac (Phase 281)
// Utility functions

use crate::settings_almanac::registry::AlmanacRegistry;

/// Format almanac registry
pub fn format_almanac_registry(registry: &AlmanacRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Almanac Registry:\n");
    output.push_str(&format!("  Almanacs: {}\n", registry.count()));
    output
}

/// Check if query is about almanac
pub fn is_almanac_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings almanac") || lower.contains("almanac settings") || lower.contains("yearly settings")
}

/// Fun fact about almanac
pub fn almanac_fun_fact() -> &'static str {
    "Anna's settings almanac chronicles your configurations throughout the year!"
}
