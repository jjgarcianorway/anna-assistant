// v0.0.659: Settings Restorer - Utilities
// Helper functions for settings restoration

/// Check if query is about restorer
pub fn is_restorer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("restorer") || lower.contains("restore settings") || lower.contains("recover settings")
}

/// Fun fact about restorer
pub fn restorer_fun_fact() -> &'static str {
    "Anna's settings restorers recover your configs from backups!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_restorer_query() {
        assert!(is_restorer_query("settings restorer"));
        assert!(!is_restorer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = restorer_fun_fact();
        assert!(fact.contains("restorer"));
    }
}
