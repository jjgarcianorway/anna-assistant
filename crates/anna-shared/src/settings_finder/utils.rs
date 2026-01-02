// v0.0.685: Finder Utilities (Phase 261)
// Utility functions for settings finder

use super::registry::FinderRegistry;

/// Format finder registry
pub fn format_finder_registry(registry: &FinderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Finder Registry:\n");
    output.push_str(&format!("  Finders: {}\n", registry.count()));
    output
}

/// Check if query is about finder
pub fn is_finder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("find settings") || lower.contains("settings finder") || lower.contains("search settings")
}

/// Fun fact about finder
pub fn finder_fun_fact() -> &'static str {
    "Anna's settings finder locates exactly the settings you need with smart scoring!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_finder_query() {
        assert!(is_finder_query("find settings"));
        assert!(!is_finder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = finder_fun_fact();
        assert!(fact.contains("finder"));
    }
}
