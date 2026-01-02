// v0.0.709: Digest Utilities (Phase 285)
// Utility functions for digest operations

use super::registry::DigestRegistry;

/// Format digest registry
pub fn format_digest_registry(registry: &DigestRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Digest Registry:\n");
    output.push_str(&format!("  Digests: {}\n", registry.count()));
    output
}

/// Check if query is about digest
pub fn is_digest_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings digest") || lower.contains("digest settings") || lower.contains("settings summary")
}

/// Fun fact about digest
pub fn digest_fun_fact() -> &'static str {
    "Anna's settings digest condenses configuration changes into easy-to-read summaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_digest_query() {
        assert!(is_digest_query("settings digest"));
        assert!(!is_digest_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = digest_fun_fact();
        assert!(fact.contains("digest"));
    }
}
