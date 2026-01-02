// v0.0.692: Settings Chronicle Helpers (Phase 268)
// Utility functions for chronicle

use super::registry::ChronicleRegistry;

/// Format chronicle registry
pub fn format_chronicle_registry(registry: &ChronicleRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Chronicle Registry:\n");
    output.push_str(&format!("  Chronicles: {}\n", registry.count()));
    output
}

/// Check if query is about chronicle
pub fn is_chronicle_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("chronicle settings") || lower.contains("settings chronicle") || lower.contains("settings changes")
}

/// Fun fact about chronicle
pub fn chronicle_fun_fact() -> &'static str {
    "Anna's settings chronicle monitors every configuration change in real-time!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chronicle_query() {
        assert!(is_chronicle_query("chronicle settings"));
        assert!(!is_chronicle_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = chronicle_fun_fact();
        assert!(fact.contains("chronicle"));
    }
}
