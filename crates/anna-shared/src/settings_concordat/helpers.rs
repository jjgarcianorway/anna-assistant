// v0.0.732: Settings Concordat Helpers (Phase 308)
// Helper functions for settings concordat

use super::core::ConcordatRegistry;

/// Format concordat registry
pub fn format_concordat_registry(registry: &ConcordatRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Concordat Registry:\n");
    output.push_str(&format!("  Concordats: {}\n", registry.count()));
    output
}

/// Check if query is about concordat
pub fn is_concordat_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings concordat") || lower.contains("concordat settings") || lower.contains("religious agreement")
}

/// Fun fact about concordat
pub fn concordat_fun_fact() -> &'static str {
    "Anna's settings concordat establishes canonical governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_concordat_query() {
        assert!(is_concordat_query("settings concordat"));
        assert!(!is_concordat_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = concordat_fun_fact();
        assert!(fact.contains("concordat"));
    }
}
