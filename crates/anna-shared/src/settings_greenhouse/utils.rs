// v0.0.770: Settings Greenhouse - Utils Module
// Utility functions for greenhouse operations

use super::registry::GreenhouseRegistry;

/// Format greenhouse registry
pub fn format_greenhouse_registry(registry: &GreenhouseRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Greenhouse Registry:\n");
    output.push_str(&format!("  Greenhouses: {}\n", registry.count()));
    output
}

/// Check if query is about greenhouse
pub fn is_greenhouse_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings greenhouse") || lower.contains("greenhouse settings") || lower.contains("controlled greenhouse")
}

/// Fun fact about greenhouse
pub fn greenhouse_fun_fact() -> &'static str {
    "Anna's settings greenhouse cultivates controlled boundaries!"
}
