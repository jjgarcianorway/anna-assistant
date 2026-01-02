// v0.0.722: Ordinance Helpers (Phase 298)
// Helper functions for ordinance operations

use super::registry::OrdinanceRegistry;

/// Format ordinance registry
pub fn format_ordinance_registry(registry: &OrdinanceRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Ordinance Registry:\n");
    output.push_str(&format!("  Ordinances: {}\n", registry.count()));
    output
}

/// Check if query is about ordinance
pub fn is_ordinance_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings ordinance") || lower.contains("ordinance settings") || lower.contains("local ordinance")
}

/// Fun fact about ordinance
pub fn ordinance_fun_fact() -> &'static str {
    "Anna's settings ordinance implements local regulations for configuration management!"
}
