// v0.0.677: Settings Reducer Utils (Phase 253)
// Utility functions for reducer operations

/// Check if query is about reducer
pub fn is_reducer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("reduce settings") || lower.contains("settings reducer") || lower.contains("aggregate settings")
}

/// Fun fact about reducer
pub fn reducer_fun_fact() -> &'static str {
    "Anna's settings reducer aggregates your settings into meaningful summaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_reducer_query() {
        assert!(is_reducer_query("reduce settings"));
        assert!(!is_reducer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reducer_fun_fact();
        assert!(fact.contains("reducer"));
    }
}
