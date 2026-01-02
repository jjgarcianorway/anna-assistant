// v0.0.744: Settings Realm Utilities (Phase 320)
// Utility functions for settings realm

use super::registry::RealmRegistry;

/// Format realm registry
pub fn format_realm_registry(registry: &RealmRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Realm Registry:\n");
    output.push_str(&format!("  Realms: {}\n", registry.count()));
    output
}

/// Check if query is about realm
pub fn is_realm_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings realm") || lower.contains("realm settings") || lower.contains("royal realm")
}

/// Fun fact about realm
pub fn realm_fun_fact() -> &'static str {
    "Anna's settings realm establishes royal sovereignty!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_realm_query() {
        assert!(is_realm_query("settings realm"));
        assert!(!is_realm_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = realm_fun_fact();
        assert!(fact.contains("realm"));
    }
}
