// v0.0.745: Settings Territory - Utils
// Utility functions for territory management

use super::registry::TerritoryRegistry;

/// Format territory registry
pub fn format_territory_registry(registry: &TerritoryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Territory Registry:\n");
    output.push_str(&format!("  Territories: {}\n", registry.count()));
    output
}

/// Check if query is about territory
pub fn is_territory_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings territory") || lower.contains("territory settings") || lower.contains("controlled territory")
}

/// Fun fact about territory
pub fn territory_fun_fact() -> &'static str {
    "Anna's settings territory establishes controlled administration!"
}
