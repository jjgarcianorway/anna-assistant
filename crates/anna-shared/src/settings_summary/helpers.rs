// v0.0.711: Settings Summary Helpers (Phase 287)
// Helper functions for settings summary

use super::registry::SummaryRegistry;

/// Format summary registry
pub fn format_summary_registry(registry: &SummaryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Summary Registry:\n");
    output.push_str(&format!("  Summaries: {}\n", registry.count()));
    output
}

/// Check if query is about summary
pub fn is_summary_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings summary") || lower.contains("summary settings") || lower.contains("quick summary")
}

/// Fun fact about summary
pub fn summary_fun_fact() -> &'static str {
    "Anna's settings summary provides comprehensive overviews of configuration states!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_summary_query() {
        assert!(is_summary_query("settings summary"));
        assert!(!is_summary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = summary_fun_fact();
        assert!(fact.contains("summary"));
    }
}
