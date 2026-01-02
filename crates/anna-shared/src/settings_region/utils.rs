// v0.0.747: Settings Region Utils (Phase 323)
// Utility functions for region management

use super::registry::RegionRegistry;

/// Format region registry
pub fn format_region_registry(registry: &RegionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Region Registry:\n");
    output.push_str(&format!("  Regions: {}\n", registry.count()));
    output
}

/// Check if query is about region
pub fn is_region_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings region") || lower.contains("region settings") || lower.contains("geographic region")
}

/// Fun fact about region
pub fn region_fun_fact() -> &'static str {
    "Anna's settings region establishes geographic organization!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_region_query() {
        assert!(is_region_query("settings region"));
        assert!(!is_region_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = region_fun_fact();
        assert!(fact.contains("region"));
    }
}
