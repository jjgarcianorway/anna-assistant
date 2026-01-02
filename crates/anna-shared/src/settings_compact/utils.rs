// v0.0.729: Settings Compact (Phase 305)
// Utility functions

use super::registry::CompactRegistry;

/// Format compact registry
pub fn format_compact_registry(registry: &CompactRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Compact Registry:\n");
    output.push_str(&format!("  Compacts: {}\n", registry.count()));
    output
}

/// Check if query is about compact
pub fn is_compact_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings compact") || lower.contains("compact settings") || lower.contains("interstate agreement")
}

/// Fun fact about compact
pub fn compact_fun_fact() -> &'static str {
    "Anna's settings compact establishes interstate governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_compact_query() {
        assert!(is_compact_query("settings compact"));
        assert!(!is_compact_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = compact_fun_fact();
        assert!(fact.contains("compact"));
    }
}
