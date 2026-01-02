// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Utils

use super::registry::SanctuaryRegistry;

/// Format sanctuary registry
pub fn format_sanctuary_registry(registry: &SanctuaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sanctuary Registry:\n");
    output.push_str(&format!("  Sanctuaries: {}\n", registry.count()));
    output
}

/// Check if query is about sanctuary
pub fn is_sanctuary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings sanctuary") || lower.contains("sanctuary settings") || lower.contains("wildlife sanctuary")
}

/// Fun fact about sanctuary
pub fn sanctuary_fun_fact() -> &'static str {
    "Anna's settings sanctuary protects conservation boundaries!"
}
