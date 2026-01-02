// v0.0.774: Settings Herbarium - Utils
// Utility functions

use super::registry::HerbariumRegistry;

/// Format herbarium registry
pub fn format_herbarium_registry(registry: &HerbariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Herbarium Registry:\n");
    output.push_str(&format!("  Herbariums: {}\n", registry.count()));
    output
}

/// Check if query is about herbarium
pub fn is_herbarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings herbarium") || lower.contains("herbarium settings") || lower.contains("plant herbarium")
}

/// Fun fact about herbarium
pub fn herbarium_fun_fact() -> &'static str {
    "Anna's settings herbarium preserves taxonomy boundaries!"
}
