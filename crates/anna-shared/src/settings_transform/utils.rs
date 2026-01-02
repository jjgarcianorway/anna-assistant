// v0.0.666: Settings Transform Utils (Phase 242)
// Utility functions for transformer

/// Check if query is about transformer
pub fn is_transformer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("transform") || lower.contains("settings transformer") || lower.contains("convert settings")
}

/// Fun fact about transformer
pub fn transformer_fun_fact() -> &'static str {
    "Anna's settings transformer converts between different formats and structures!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_transformer_query() {
        assert!(is_transformer_query("transform settings"));
        assert!(!is_transformer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = transformer_fun_fact();
        assert!(fact.contains("transform"));
    }
}
