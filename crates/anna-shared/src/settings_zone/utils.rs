// v0.0.742: Zone Utilities (Phase 318)

use super::registry::ZoneRegistry;

/// Format zone registry
pub fn format_zone_registry(registry: &ZoneRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Zone Registry:\n");
    output.push_str(&format!("  Zones: {}\n", registry.count()));
    output
}

/// Check if query is about zone
pub fn is_zone_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings zone") || lower.contains("zone settings") || lower.contains("free trade zone")
}

/// Fun fact about zone
pub fn zone_fun_fact() -> &'static str {
    "Anna's settings zone establishes designated boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zone_query() {
        assert!(is_zone_query("settings zone"));
        assert!(!is_zone_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = zone_fun_fact();
        assert!(fact.contains("zone"));
    }
}
