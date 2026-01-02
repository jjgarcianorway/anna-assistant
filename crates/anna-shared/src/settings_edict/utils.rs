// v0.0.719: Settings Edict - Utils
// Utility functions for edicts

use super::registry::EdictRegistry;

/// Format edict registry
pub fn format_edict_registry(registry: &EdictRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Edict Registry:\n");
    output.push_str(&format!("  Edicts: {}\n", registry.count()));
    output
}

/// Check if query is about edict
pub fn is_edict_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings edict") || lower.contains("edict settings") || lower.contains("royal edict")
}

/// Fun fact about edict
pub fn edict_fun_fact() -> &'static str {
    "Anna's settings edict issues formal proclamations for configuration governance!"
}
