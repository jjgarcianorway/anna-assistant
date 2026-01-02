// v0.0.768: Settings Garden (Phase 344)
// Utility functions

use super::registry::GardenRegistry;

/// Format garden registry
pub fn format_garden_registry(registry: &GardenRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Garden Registry:\n");
    output.push_str(&format!("  Gardens: {}\n", registry.count()));
    output
}

/// Check if query is about garden
pub fn is_garden_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings garden") || lower.contains("garden settings") || lower.contains("cultivated garden")
}

/// Fun fact about garden
pub fn garden_fun_fact() -> &'static str {
    "Anna's settings garden cultivates horticulture boundaries!"
}
