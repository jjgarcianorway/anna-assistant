// v0.0.725: Constitution Helper Functions (Phase 301)

use super::registry::ConstitutionRegistry;

/// Format constitution registry
pub fn format_constitution_registry(registry: &ConstitutionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Constitution Registry:\n");
    output.push_str(&format!("  Constitutions: {}\n", registry.count()));
    output
}

/// Check if query is about constitution
pub fn is_constitution_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings constitution") || lower.contains("constitution settings") || lower.contains("supreme law")
}

/// Fun fact about constitution
pub fn constitution_fun_fact() -> &'static str {
    "Anna's settings constitution establishes supreme governance principles!"
}
