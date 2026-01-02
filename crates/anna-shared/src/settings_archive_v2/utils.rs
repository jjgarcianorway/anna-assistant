// v0.0.702: Settings Archive V2 (Phase 278)
// Utility functions

use super::registry::ArchiveRegistryV2;

/// Format archive registry v2
pub fn format_archive_registry_v2(registry: &ArchiveRegistryV2) -> String {
    let mut output = String::new();
    output.push_str("Settings Archive V2 Registry:\n");
    output.push_str(&format!("  Archives: {}\n", registry.count()));
    output
}

/// Check if query is about archive v2
pub fn is_archive_v2_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings archive") || lower.contains("archive settings") || lower.contains("long-term storage")
}

/// Fun fact about archive v2
pub fn archive_v2_fun_fact() -> &'static str {
    "Anna's settings archive v2 preserves your configurations for long-term storage!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_archive_v2_query() {
        assert!(is_archive_v2_query("settings archive"));
        assert!(!is_archive_v2_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = archive_v2_fun_fact();
        assert!(fact.contains("archive"));
    }
}
