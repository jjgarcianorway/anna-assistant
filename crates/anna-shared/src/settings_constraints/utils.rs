// v0.0.570: Constraint Utilities (Phase 146)
// Helper functions for constraint handling

/// Check if query is about constraints
pub fn is_constraint_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("constraint")
        || lower.contains("rule")
        || lower.contains("settings conflict")
        || lower.contains("validate settings")
}

/// Fun fact about constraints
pub fn constraint_fun_fact() -> &'static str {
    "Settings constraints help prevent configuration conflicts before they cause problems!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_constraint_query() {
        assert!(is_constraint_query("check constraints"));
        assert!(is_constraint_query("settings conflict"));
        assert!(!is_constraint_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = constraint_fun_fact();
        assert!(fact.contains("constraint"));
    }
}
