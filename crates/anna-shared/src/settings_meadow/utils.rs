// v0.0.763: Settings Meadow Utilities
// Helper functions for meadow operations

use super::meadow::MeadowRegistry;

/// Format meadow registry
pub fn format_meadow_registry(registry: &MeadowRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Meadow Registry:\n");
    output.push_str(&format!("  Meadows: {}\n", registry.count()));
    output
}

/// Check if query is about meadow
pub fn is_meadow_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings meadow") || lower.contains("meadow settings") || lower.contains("grassland meadow")
}

/// Fun fact about meadow
pub fn meadow_fun_fact() -> &'static str {
    "Anna's settings meadow establishes grazing boundaries!"
}
