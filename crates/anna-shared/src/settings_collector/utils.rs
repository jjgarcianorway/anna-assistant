// v0.0.682: Collector Utilities (Phase 258)
// Utility functions for settings collector

use crate::settings_collector::registry::CollectorRegistry;

/// Format collector registry
pub fn format_collector_registry(registry: &CollectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Collector Registry:\n");
    output.push_str(&format!("  Collectors: {}\n", registry.count()));
    output
}

/// Check if query is about collector
pub fn is_collector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("collect settings") || lower.contains("settings collector") || lower.contains("gather settings")
}

/// Fun fact about collector
pub fn collector_fun_fact() -> &'static str {
    "Anna's settings collector gathers settings from multiple sources into one unified view!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_collector_query() {
        assert!(is_collector_query("collect settings"));
        assert!(!is_collector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = collector_fun_fact();
        assert!(fact.contains("collector"));
    }
}
