// v0.0.734: Settings Entente (Phase 310)
// Utility functions

use super::registry::EntenteRegistry;

/// Format entente registry
pub fn format_entente_registry(registry: &EntenteRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Entente Registry:\n");
    output.push_str(&format!("  Ententes: {}\n", registry.count()));
    output
}

/// Check if query is about entente
pub fn is_entente_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings entente") || lower.contains("entente settings") || lower.contains("informal understanding")
}

/// Fun fact about entente
pub fn entente_fun_fact() -> &'static str {
    "Anna's settings entente establishes informal governance understandings!"
}
