// v0.0.733: Settings Convention Utils (Phase 309)
// Utility functions for convention queries

/// Check if query is about convention
pub fn is_convention_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings convention") || lower.contains("convention settings") || lower.contains("formal gathering")
}

/// Fun fact about convention
pub fn convention_fun_fact() -> &'static str {
    "Anna's settings convention establishes formal governance standards!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_convention_query() {
        assert!(is_convention_query("settings convention"));
        assert!(!is_convention_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = convention_fun_fact();
        assert!(fact.contains("convention"));
    }
}
