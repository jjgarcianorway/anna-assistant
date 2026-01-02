// v0.0.676: Settings Grouper - Utilities (Phase 252)
// Utility functions for settings grouper

/// Check if query is about grouper
pub fn is_grouper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("group settings") || lower.contains("settings grouper") || lower.contains("categorize settings")
}

/// Fun fact about grouper
pub fn grouper_fun_fact() -> &'static str {
    "Anna's settings grouper organizes your settings into logical categories!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_grouper_query() {
        assert!(is_grouper_query("group settings"));
        assert!(!is_grouper_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = grouper_fun_fact();
        assert!(fact.contains("grouper"));
    }
}
