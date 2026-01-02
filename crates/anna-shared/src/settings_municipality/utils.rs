// v0.0.750: Settings Municipality Utils (Phase 326)
// Utility functions for municipality

use super::registry::MunicipalityRegistry;

/// Format municipality registry
pub fn format_municipality_registry(registry: &MunicipalityRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Municipality Registry:\n");
    output.push_str(&format!("  Municipalities: {}\n", registry.count()));
    output
}

/// Check if query is about municipality
pub fn is_municipality_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings municipality") || lower.contains("municipality settings") || lower.contains("municipal corporation")
}

/// Fun fact about municipality
pub fn municipality_fun_fact() -> &'static str {
    "Anna's settings municipality establishes municipal self-governance!"
}
