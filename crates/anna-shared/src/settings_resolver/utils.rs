// v0.0.599: Settings Resolver Utilities (Phase 175)
// Utility functions for resolver

use super::resolver::SettingsResolver;

/// Format resolver
pub fn format_resolver(resolver: &SettingsResolver) -> String {
    let mut output = String::new();
    output.push_str("Settings Resolver:\n");
    output.push_str(&format!("  Dependencies: {}\n", resolver.dependency_count()));
    output.push_str(&format!("  Conflicts: {}\n", resolver.conflict_count()));
    output.push_str(&format!("  Resolutions: {}\n", resolver.resolution_count()));
    output
}

/// Check if query is about resolver
pub fn is_resolver_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("resolve")
        || lower.contains("conflict")
        || lower.contains("dependency")
}

/// Fun fact about resolver
pub fn resolver_fun_fact() -> &'static str {
    "Anna automatically resolves settings conflicts based on configurable strategies!"
}
