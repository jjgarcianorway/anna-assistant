// v0.0.670: Settings Query Engine Utils (Phase 246)
// Utility functions for query engine

/// Check if query is about query engine
pub fn is_query_engine_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("query engine") || lower.contains("query settings") || lower.contains("search engine")
}

/// Fun fact about query engine
pub fn query_engine_fun_fact() -> &'static str {
    "Anna's query engine supports complex queries with multiple conditions!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_query_engine_query() {
        assert!(is_query_engine_query("query settings"));
        assert!(!is_query_engine_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = query_engine_fun_fact();
        assert!(fact.contains("query"));
    }
}
