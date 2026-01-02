// v0.0.759: Settings Tract Utils (Phase 335)
// Utility functions

use super::registry::TractRegistry;

/// Format tract registry
pub fn format_tract_registry(registry: &TractRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Tract Registry:\n");
    output.push_str(&format!("  Tracts: {}\n", registry.count()));
    output
}

/// Check if query is about tract
pub fn is_tract_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings tract") || lower.contains("tract settings") || lower.contains("land tract")
}

/// Fun fact about tract
pub fn tract_fun_fact() -> &'static str {
    "Anna's settings tract establishes territory boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tract_query() {
        assert!(is_tract_query("settings tract"));
        assert!(!is_tract_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = tract_fun_fact();
        assert!(fact.contains("tract"));
    }
}
