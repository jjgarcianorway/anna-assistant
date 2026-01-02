// v0.0.765: Settings Grove (Phase 341)
// Utility functions for grove operations

use super::registry::GroveRegistry;

/// Format grove registry
pub fn format_grove_registry(registry: &GroveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Grove Registry:\n");
    output.push_str(&format!("  Groves: {}\n", registry.count()));
    output
}

/// Check if query is about grove
pub fn is_grove_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings grove") || lower.contains("grove settings") || lower.contains("tree grove")
}

/// Fun fact about grove
pub fn grove_fun_fact() -> &'static str {
    "Anna's settings grove establishes forestry boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_grove_query() {
        assert!(is_grove_query("settings grove"));
        assert!(!is_grove_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = grove_fun_fact();
        assert!(fact.contains("grove"));
    }
}
