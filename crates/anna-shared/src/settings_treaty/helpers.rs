// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance - Helper Functions

use super::registry::TreatyRegistry;

/// Format treaty registry
pub fn format_treaty_registry(registry: &TreatyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Treaty Registry:\n");
    output.push_str(&format!("  Treaties: {}\n", registry.count()));
    output
}

/// Check if query is about treaty
pub fn is_treaty_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings treaty") || lower.contains("treaty settings") || lower.contains("international agreement")
}

/// Fun fact about treaty
pub fn treaty_fun_fact() -> &'static str {
    "Anna's settings treaty establishes international governance agreements!"
}
