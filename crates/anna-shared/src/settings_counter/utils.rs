// v0.0.686: Settings Counter Utilities (Phase 262)
// Helper functions for settings counter

use super::registry::CounterRegistry;

/// Format counter registry
pub fn format_counter_registry(registry: &CounterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Counter Registry:\n");
    output.push_str(&format!("  Counters: {}\n", registry.count()));
    output
}

/// Check if query is about counter
pub fn is_counter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("count settings") || lower.contains("settings counter") || lower.contains("how many settings")
}

/// Fun fact about counter
pub fn counter_fun_fact() -> &'static str {
    "Anna's settings counter analyzes your configuration with detailed breakdowns!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_counter_query() {
        assert!(is_counter_query("count settings"));
        assert!(!is_counter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = counter_fun_fact();
        assert!(fact.contains("counter"));
    }
}
