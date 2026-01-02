// v0.0.764: Settings Pasture - Utilities (Phase 340)

/// Check if query is about pasture
pub fn is_pasture_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings pasture") || lower.contains("pasture settings") || lower.contains("grazing pasture")
}

/// Fun fact about pasture
pub fn pasture_fun_fact() -> &'static str {
    "Anna's settings pasture establishes livestock boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pasture_query() {
        assert!(is_pasture_query("settings pasture"));
        assert!(!is_pasture_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = pasture_fun_fact();
        assert!(fact.contains("pasture"));
    }
}
