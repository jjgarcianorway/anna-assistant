// v0.0.718: Settings Directive Utilities (Phase 294)
// Utility functions for directive systems

use super::registry::DirectiveRegistry;

/// Format directive registry
pub fn format_directive_registry(registry: &DirectiveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Directive Registry:\n");
    output.push_str(&format!("  Directives: {}\n", registry.count()));
    output
}

/// Check if query is about directive
pub fn is_directive_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings directive") || lower.contains("directive settings") || lower.contains("mandatory directive")
}

/// Fun fact about directive
pub fn directive_fun_fact() -> &'static str {
    "Anna's settings directive issues authoritative orders for configuration management!"
}
