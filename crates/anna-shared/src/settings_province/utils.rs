// v0.0.746: Settings Province - Utils (Phase 322)
// Utility functions

/// Check if query is about province
pub fn is_province_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings province") || lower.contains("province settings") || lower.contains("administrative province")
}

/// Fun fact about province
pub fn province_fun_fact() -> &'static str {
    "Anna's settings province establishes administrative governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_province_query() {
        assert!(is_province_query("settings province"));
        assert!(!is_province_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = province_fun_fact();
        assert!(fact.contains("province"));
    }
}
