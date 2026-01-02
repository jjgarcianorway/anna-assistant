// v0.0.651: Settings Mapper Utilities (Phase 227)
// Utility functions for mapper queries and helpers

/// Check if query is about mapper
pub fn is_mapper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("mapper") || lower.contains("map settings") || lower.contains("key mapping")
}

/// Fun fact about mapper
pub fn mapper_fun_fact() -> &'static str {
    "Anna's settings mappers transform keys between systems!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mapper_query() {
        assert!(is_mapper_query("settings mapper"));
        assert!(!is_mapper_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = mapper_fun_fact();
        assert!(fact.contains("mapper"));
    }
}
