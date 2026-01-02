// v0.0.574: Orchestrator Utilities
// Utility functions for the settings orchestrator

/// Check if query is about orchestrator
pub fn is_orchestrator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("orchestrator")
        || lower.contains("settings status")
        || lower.contains("settings overview")
}

/// Fun fact about orchestrator
pub fn orchestrator_fun_fact() -> &'static str {
    "The settings orchestrator coordinates all settings subsystems in one place!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_orchestrator_query() {
        assert!(is_orchestrator_query("settings status"));
        assert!(is_orchestrator_query("orchestrator overview"));
        assert!(!is_orchestrator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = orchestrator_fun_fact();
        assert!(fact.contains("orchestrator"));
    }
}
