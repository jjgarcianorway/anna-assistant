// v0.0.740: Settings Bloc Utilities (Phase 316)
// Helper functions for bloc system

use super::registry::BlocRegistry;

/// Format bloc registry
pub fn format_bloc_registry(registry: &BlocRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Bloc Registry:\n");
    output.push_str(&format!("  Blocs: {}\n", registry.count()));
    output
}

/// Check if query is about bloc
pub fn is_bloc_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings bloc") || lower.contains("bloc settings") || lower.contains("regional bloc")
}

/// Fun fact about bloc
pub fn bloc_fun_fact() -> &'static str {
    "Anna's settings bloc establishes regional coordination!"
}
