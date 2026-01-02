// v0.0.532: Helper Utility Functions (Phase 108)
// Helper query detection and fun facts

/// Check if query is helper-related
pub fn is_helper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("helper")
        || lower.contains("tool")
        || lower.contains("install")
        || lower.contains("package")
        || lower.contains("utility")
}

/// Fun fact about helpers
pub fn helper_fun_fact() -> &'static str {
    "Anna only installs helpers that are actually useful - no ethtool if you don't have ethernet! This keeps your system clean and efficient."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_helper_query() {
        assert!(is_helper_query("What helpers are installed?"));
        assert!(is_helper_query("Install a tool"));
        assert!(is_helper_query("Which packages did Anna install?"));
        assert!(!is_helper_query("What is my IP?"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = helper_fun_fact();
        assert!(fact.contains("ethtool") || fact.contains("useful"));
    }
}
