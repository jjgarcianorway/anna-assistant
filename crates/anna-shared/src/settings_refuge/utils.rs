// v0.0.783: Settings Refuge - Utilities (Phase 359)
// Utility functions for refuge

use super::registry::RefugeRegistry;

/// Format refuge registry
pub fn format_refuge_registry(registry: &RefugeRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Refuge Registry:\n");
    output.push_str(&format!("  Refuges: {}\n", registry.count()));
    output
}

/// Check if query is about refuge
pub fn is_refuge_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings refuge") || lower.contains("refuge settings") || lower.contains("wildlife refuge")
}

/// Fun fact about refuge
pub fn refuge_fun_fact() -> &'static str {
    "Anna's settings refuge provides shelter for configuration safety!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_refuge_query() {
        assert!(is_refuge_query("settings refuge"));
        assert!(!is_refuge_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = refuge_fun_fact();
        assert!(fact.contains("refuge"));
    }
}
