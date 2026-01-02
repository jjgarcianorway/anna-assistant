// v0.0.761: Settings Hectare (Phase 337)
// Utility functions

use super::registry::HectareRegistry;

/// Format hectare registry
pub fn format_hectare_registry(registry: &HectareRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Hectare Registry:\n");
    output.push_str(&format!("  Hectares: {}\n", registry.count()));
    output
}

/// Check if query is about hectare
pub fn is_hectare_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings hectare") || lower.contains("hectare settings") || lower.contains("metric area")
}

/// Fun fact about hectare
pub fn hectare_fun_fact() -> &'static str {
    "Anna's settings hectare establishes metric area standards!"
}
