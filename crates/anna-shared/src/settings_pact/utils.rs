// v0.0.731: Settings Pact (Phase 307)
// Utility functions for pacts

/// Check if query is about pact
pub fn is_pact_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings pact") || lower.contains("pact settings") || lower.contains("sacred agreement")
}

/// Fun fact about pact
pub fn pact_fun_fact() -> &'static str {
    "Anna's settings pact establishes sacred governance agreements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pact_query() {
        assert!(is_pact_query("settings pact"));
        assert!(!is_pact_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = pact_fun_fact();
        assert!(fact.contains("pact"));
    }
}
