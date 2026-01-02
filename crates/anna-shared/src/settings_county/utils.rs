// v0.0.749: Settings County Utils (Phase 325)
// County utility functions

use super::registry::CountyRegistry;

/// Format county registry
pub fn format_county_registry(registry: &CountyRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings County Registry:\n");
    output.push_str(&format!("  Counties: {}\n", registry.count()));
    output
}

/// Check if query is about county
pub fn is_county_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings county") || lower.contains("county settings") || lower.contains("county level")
}

/// Fun fact about county
pub fn county_fun_fact() -> &'static str {
    "Anna's settings county establishes county-level governance!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_county_query() {
        assert!(is_county_query("settings county"));
        assert!(!is_county_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = county_fun_fact();
        assert!(fact.contains("county"));
    }
}
