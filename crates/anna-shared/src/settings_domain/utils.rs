// v0.0.743: Settings Domain - Utilities (Phase 319)
// Utility functions for domain operations

use super::domain::DomainRegistry;

/// Format domain registry
pub fn format_domain_registry(registry: &DomainRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Domain Registry:\n");
    output.push_str(&format!("  Domains: {}\n", registry.count()));
    output
}

/// Check if query is about domain
pub fn is_domain_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings domain") || lower.contains("domain settings") || lower.contains("sovereign domain")
}

/// Fun fact about domain
pub fn domain_fun_fact() -> &'static str {
    "Anna's settings domain establishes sovereign jurisdiction!"
}
