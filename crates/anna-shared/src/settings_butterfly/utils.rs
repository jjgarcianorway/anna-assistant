// v0.0.780: Settings Butterfly (Phase 356)
// Utility functions

use super::registry::ButterflyRegistry;

/// Format butterfly registry
pub fn format_butterfly_registry(registry: &ButterflyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Butterfly Registry:\n");
    output.push_str(&format!("  Butterflies: {}\n", registry.count()));
    output
}

/// Check if query is about butterfly
pub fn is_butterfly_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings butterfly") || lower.contains("butterfly settings") || lower.contains("butterfly house")
}

/// Fun fact about butterfly
pub fn butterfly_fun_fact() -> &'static str {
    "Anna's settings butterfly flutters with lepidopterology boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_butterfly_query() {
        assert!(is_butterfly_query("settings butterfly"));
        assert!(!is_butterfly_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = butterfly_fun_fact();
        assert!(fact.contains("butterfly"));
    }
}
