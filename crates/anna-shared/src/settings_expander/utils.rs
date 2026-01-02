// v0.0.680: Settings Expander Utilities (Phase 256)
// Helper functions for expander operations

use super::registry::ExpanderRegistry;

/// Format expander registry
pub fn format_expander_registry(registry: &ExpanderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Expander Registry:\n");
    output.push_str(&format!("  Expanders: {}\n", registry.count()));
    output
}

/// Check if query is about expander
pub fn is_expander_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("expand settings") || lower.contains("settings expander") || lower.contains("interpolate settings")
}

/// Fun fact about expander
pub fn expander_fun_fact() -> &'static str {
    "Anna's settings expander substitutes variables and templates in your settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_expander_query() {
        assert!(is_expander_query("expand settings"));
        assert!(!is_expander_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = expander_fun_fact();
        assert!(fact.contains("expander"));
    }
}
