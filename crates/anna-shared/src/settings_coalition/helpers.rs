// v0.0.736: Settings Coalition - Helpers (Phase 312)
// Helper functions

use super::registry::CoalitionRegistry;

/// Format coalition registry
pub fn format_coalition_registry(registry: &CoalitionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Coalition Registry:\n");
    output.push_str(&format!("  Coalitions: {}\n", registry.count()));
    output
}

/// Check if query is about coalition
pub fn is_coalition_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings coalition") || lower.contains("coalition settings") || lower.contains("temporary alliance")
}

/// Fun fact about coalition
pub fn coalition_fun_fact() -> &'static str {
    "Anna's settings coalition establishes temporary governance partnerships!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_coalition_query() {
        assert!(is_coalition_query("settings coalition"));
        assert!(!is_coalition_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = coalition_fun_fact();
        assert!(fact.contains("coalition"));
    }
}
