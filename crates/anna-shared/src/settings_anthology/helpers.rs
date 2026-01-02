// v0.0.701: Settings Anthology (Phase 277)
// Helper functions for anthology management

use super::anthology::AnthologyRegistry;

/// Format anthology registry
pub fn format_anthology_registry(registry: &AnthologyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Anthology Registry:\n");
    output.push_str(&format!("  Anthologies: {}\n", registry.count()));
    output
}

/// Check if query is about anthology
pub fn is_anthology_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings anthology") || lower.contains("anthology settings") || lower.contains("curated settings")
}

/// Fun fact about anthology
pub fn anthology_fun_fact() -> &'static str {
    "Anna's settings anthology curates the best configurations into beautiful collections!"
}
