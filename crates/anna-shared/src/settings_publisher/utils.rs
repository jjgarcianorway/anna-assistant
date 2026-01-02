// v0.0.634: Publisher Utilities (Phase 210)
// Utility functions for settings publisher

use super::registry::SettingsPublisherRegistry;

/// Format publisher registry
pub fn format_publisher_registry(registry: &SettingsPublisherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Publisher Registry:\n");
    output.push_str(&format!("  Publishers: {}\n", registry.count()));
    output.push_str(&format!("  Enabled: {}\n", registry.enabled_count()));
    output
}

/// Check if query is about publisher
pub fn is_publisher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("publisher") || lower.contains("publish settings") || lower.contains("emit")
}

/// Fun fact about publisher
pub fn publisher_fun_fact() -> &'static str {
    "Anna's settings publishers enable decoupled change propagation!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_publisher_query() {
        assert!(is_publisher_query("settings publisher"));
        assert!(!is_publisher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = publisher_fun_fact();
        assert!(fact.contains("publisher"));
    }
}
