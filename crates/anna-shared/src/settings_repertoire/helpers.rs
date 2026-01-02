// v0.0.703: Settings Repertoire Helpers (Phase 279)
// Helper functions for repertoire management

use super::registry::RepertoireRegistry;

/// Format repertoire registry
pub fn format_repertoire_registry(registry: &RepertoireRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Repertoire Registry:\n");
    output.push_str(&format!("  Repertoires: {}\n", registry.count()));
    output
}

/// Check if query is about repertoire
pub fn is_repertoire_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings repertoire") || lower.contains("repertoire settings") || lower.contains("available settings")
}

/// Fun fact about repertoire
pub fn repertoire_fun_fact() -> &'static str {
    "Anna's settings repertoire performs your configurations with virtuoso precision!"
}
