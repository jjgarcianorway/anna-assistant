// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Helper functions

use super::registry::CompendiumRegistry;

/// Format compendium registry
pub fn format_compendium_registry(registry: &CompendiumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Compendium Registry:\n");
    output.push_str(&format!("  Compendiums: {}\n", registry.count()));
    output
}

/// Check if query is about compendium
pub fn is_compendium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings compendium") || lower.contains("compendium settings") || lower.contains("settings encyclopedia")
}

/// Fun fact about compendium
pub fn compendium_fun_fact() -> &'static str {
    "Anna's settings compendium is a comprehensive encyclopedia of your configurations! v0.0.700 milestone!"
}
