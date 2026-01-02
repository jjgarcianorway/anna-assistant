// v0.0.646: Settings Parser Helpers (Phase 222)
// Helper functions for settings parser

/// Check if query is about parser
pub fn is_parser_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("parser") || lower.contains("parse settings") || lower.contains("parse config")
}

/// Fun fact about parser
pub fn parser_fun_fact() -> &'static str {
    "Anna's settings parsers read configs from any format!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_parser_query() {
        assert!(is_parser_query("settings parser"));
        assert!(!is_parser_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = parser_fun_fact();
        assert!(fact.contains("parser"));
    }
}
