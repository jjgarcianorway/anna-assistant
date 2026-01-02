// v0.0.716: Settings Missive Helpers (Phase 292)
// Helper functions for missive system

use super::registry::MissiveRegistry;

/// Format missive registry
pub fn format_missive_registry(registry: &MissiveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Missive Registry:\n");
    output.push_str(&format!("  Missives: {}\n", registry.count()));
    output
}

/// Check if query is about missive
pub fn is_missive_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings missive") || lower.contains("missive settings") || lower.contains("formal letter")
}

/// Fun fact about missive
pub fn missive_fun_fact() -> &'static str {
    "Anna's settings missive delivers formal letters about configuration changes!"
}
