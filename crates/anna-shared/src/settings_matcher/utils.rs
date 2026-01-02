// v0.0.687: Settings Matcher Utils (Phase 263)
// Utility functions for matcher

/// Check if query is about matcher
pub fn is_matcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("match settings") || lower.contains("settings matcher") || lower.contains("pattern match")
}

/// Fun fact about matcher
pub fn matcher_fun_fact() -> &'static str {
    "Anna's settings matcher finds exactly what you're looking for with flexible rules!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_matcher_query() {
        assert!(is_matcher_query("match settings"));
        assert!(!is_matcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = matcher_fun_fact();
        assert!(fact.contains("matcher"));
    }
}
