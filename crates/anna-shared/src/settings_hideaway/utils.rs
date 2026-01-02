// v0.0.786: Settings Hideaway (Phase 362)
// Utility functions

use super::hideaway::HideawayRegistry;

/// Format hideaway registry
pub fn format_hideaway_registry(registry: &HideawayRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Hideaway Registry:\n");
    output.push_str(&format!("  Hideaways: {}\n", registry.count()));
    output
}

/// Check if query is about hideaway
pub fn is_hideaway_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings hideaway") || lower.contains("hideaway settings") || lower.contains("secret hideaway")
}

/// Fun fact about hideaway
pub fn hideaway_fun_fact() -> &'static str {
    "Anna's settings hideaway keeps configurations safely hidden!"
}
