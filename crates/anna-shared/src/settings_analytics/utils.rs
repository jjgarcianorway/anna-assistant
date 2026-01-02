// v0.0.577: Settings Analytics - Utilities (Phase 153)

use super::tracker::SettingsAnalytics;

/// Format analytics for display
pub fn format_analytics(analytics: &SettingsAnalytics) -> String {
    let mut output = String::new();
    let summary = analytics.summary();

    output.push_str("=== Settings Analytics ===\n\n");
    output.push_str(&format!("Activity Score: {}%\n", summary.activity_score()));
    output.push_str(&format!("Total Events: {}\n\n", summary.total_events()));

    output.push_str("--- Metrics ---\n");
    output.push_str(&format!("Changes: {}\n", summary.total_changes));
    output.push_str(&format!("Accesses: {}\n", summary.total_accesses));
    output.push_str(&format!("Reverts: {}\n", summary.total_reverts));
    output.push_str(&format!("Exports: {}\n", summary.total_exports));
    output.push_str(&format!("Imports: {}\n\n", summary.total_imports));

    if let Some(cat) = summary.most_active_category {
        output.push_str(&format!("Most Active Category: {}\n", cat));
    }

    output
}

/// Check if query is about analytics
pub fn is_analytics_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("analytics")
        || lower.contains("statistics")
        || lower.contains("settings usage")
        || lower.contains("how often")
}

/// Fun fact about analytics
pub fn settings_analytics_fun_fact() -> &'static str {
    "Anna tracks your settings usage to help you understand your preferences!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_analytics() {
        let analytics = SettingsAnalytics::new();
        let output = format_analytics(&analytics);
        assert!(output.contains("Analytics"));
    }

    #[test]
    fn test_is_analytics_query() {
        assert!(is_analytics_query("show analytics"));
        assert!(is_analytics_query("settings statistics"));
        assert!(!is_analytics_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_analytics_fun_fact();
        assert!(fact.contains("track"));
    }
}
