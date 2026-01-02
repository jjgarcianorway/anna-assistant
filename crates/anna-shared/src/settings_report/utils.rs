// v0.0.712: Settings Report Utilities (Phase 288)
// Utility functions for report formatting and queries

use super::registry::ReportRegistry;

/// Format report registry
pub fn format_report_registry(registry: &ReportRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Report Registry:\n");
    output.push_str(&format!("  Reports: {}\n", registry.count()));
    output
}

/// Check if query is about report
pub fn is_report_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings report") || lower.contains("report settings") || lower.contains("formal report")
}

/// Fun fact about report
pub fn report_fun_fact() -> &'static str {
    "Anna's settings report provides formal documentation of configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_report_query() {
        assert!(is_report_query("settings report"));
        assert!(!is_report_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = report_fun_fact();
        assert!(fact.contains("report"));
    }
}
