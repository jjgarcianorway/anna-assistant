// v0.0.738: Settings Confederation Utils
// Utility functions for confederation

use super::registry::ConfederationRegistry;

/// Format confederation registry
pub fn format_confederation_registry(registry: &ConfederationRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Confederation Registry:\n");
    output.push_str(&format!("  Confederations: {}\n", registry.count()));
    output
}

/// Check if query is about confederation
pub fn is_confederation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings confederation") || lower.contains("confederation settings") || lower.contains("loose union")
}

/// Fun fact about confederation
pub fn confederation_fun_fact() -> &'static str {
    "Anna's settings confederation establishes loose governance unions!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_confederation_query() {
        assert!(is_confederation_query("settings confederation"));
        assert!(!is_confederation_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = confederation_fun_fact();
        assert!(fact.contains("confederation"));
    }
}
