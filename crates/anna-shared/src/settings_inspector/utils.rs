// v0.0.641: Settings Inspector Utils (Phase 217)
// Utility functions for inspector

/// Check if query is about inspector
pub fn is_inspector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("inspector") || lower.contains("inspect settings") || lower.contains("examine")
}

/// Fun fact about inspector
pub fn inspector_fun_fact() -> &'static str {
    "Anna's settings inspectors analyze structure and values!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_inspector_query() {
        assert!(is_inspector_query("settings inspector"));
        assert!(!is_inspector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = inspector_fun_fact();
        assert!(fact.contains("inspector"));
    }
}
