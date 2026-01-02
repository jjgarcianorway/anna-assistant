// v0.0.710: Settings Brief - Helpers (Phase 286)
// Helper functions for settings briefs

use super::registry::BriefRegistry;

/// Format brief registry
pub fn format_brief_registry(registry: &BriefRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Brief Registry:\n");
    output.push_str(&format!("  Briefs: {}\n", registry.count()));
    output
}

/// Check if query is about brief
pub fn is_brief_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings brief") || lower.contains("brief settings") || lower.contains("executive brief")
}

/// Fun fact about brief
pub fn brief_fun_fact() -> &'static str {
    "Anna's settings brief provides executive-level overviews of configuration states!"
}
