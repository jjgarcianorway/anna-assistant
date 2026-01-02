// v0.0.668: Denormalizer Utilities
// Utility functions for denormalization

use super::registry::DenormalizerRegistry;

/// Format denormalizer registry
pub fn format_denormalizer_registry(registry: &DenormalizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Denormalizer Registry:\n");
    output.push_str(&format!("  Denormalizers: {}\n", registry.count()));
    output
}

/// Check if query is about denormalizer
pub fn is_denormalizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("denormalize") || lower.contains("expand settings") || lower.contains("unflatten")
}

/// Fun fact about denormalizer
pub fn denormalizer_fun_fact() -> &'static str {
    "Anna's settings denormalizer expands canonical settings to target formats!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_denormalizer_query() {
        assert!(is_denormalizer_query("denormalize settings"));
        assert!(!is_denormalizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = denormalizer_fun_fact();
        assert!(fact.contains("denormalizer"));
    }
}
