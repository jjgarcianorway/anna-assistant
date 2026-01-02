// v0.0.779: Settings Apiary - Utils (Phase 355)
// Utility functions for apiary

use super::registry::ApiaryRegistry;

/// Format apiary registry
pub fn format_apiary_registry(registry: &ApiaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Apiary Registry:\n");
    output.push_str(&format!("  Apiaries: {}\n", registry.count()));
    output
}

/// Check if query is about apiary
pub fn is_apiary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings apiary") || lower.contains("apiary settings") || lower.contains("bee apiary")
}

/// Fun fact about apiary
pub fn apiary_fun_fact() -> &'static str {
    "Anna's settings apiary buzzes with apiculture boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_apiary_query() {
        assert!(is_apiary_query("settings apiary"));
        assert!(!is_apiary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = apiary_fun_fact();
        assert!(fact.contains("apiary"));
    }
}
