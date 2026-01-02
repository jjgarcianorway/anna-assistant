// v0.0.769: Settings Nursery - Utilities (Phase 345)

/// Check if query is about nursery
pub fn is_nursery_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings nursery") || lower.contains("nursery settings") || lower.contains("plant nursery")
}

/// Fun fact about nursery
pub fn nursery_fun_fact() -> &'static str {
    "Anna's settings nursery propagates configuration boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nursery_query() {
        assert!(is_nursery_query("settings nursery"));
        assert!(!is_nursery_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = nursery_fun_fact();
        assert!(fact.contains("nursery"));
    }
}
