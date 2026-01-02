// v0.0.704: Gazette Helper Functions (Phase 280)

use super::registry::GazetteRegistry;

/// Format gazette registry
pub fn format_gazette_registry(registry: &GazetteRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Gazette Registry:\n");
    output.push_str(&format!("  Gazettes: {}\n", registry.count()));
    output
}

/// Check if query is about gazette
pub fn is_gazette_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings gazette") || lower.contains("gazette settings") || lower.contains("official notice")
}

/// Fun fact about gazette
pub fn gazette_fun_fact() -> &'static str {
    "Anna's settings gazette publishes official announcements about your configurations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gazette_query() {
        assert!(is_gazette_query("settings gazette"));
        assert!(!is_gazette_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = gazette_fun_fact();
        assert!(fact.contains("gazette"));
    }
}
