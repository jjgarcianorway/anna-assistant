// v0.0.579: Settings Dashboard Formatting (Phase 155)
// Formatting and utility functions

use super::dashboard::SettingsDashboard;

/// Format dashboard for display
pub fn format_dashboard(dashboard: &SettingsDashboard) -> String {
    let mut output = String::new();
    let stats = dashboard.stats();

    output.push_str("=== Settings Dashboard ===\n\n");
    output.push_str(&format!("Health: {} ({}%)\n", stats.overall_health, stats.health_score()));
    output.push_str(&format!("Settings: {}\n", stats.total_settings));
    output.push_str(&format!("Categories: {} ({} healthy)\n", stats.categories_count, stats.healthy_categories));
    output.push_str(&format!("Customization: {:.1}%\n\n", stats.customization_percent()));

    output.push_str("--- Categories ---\n");
    for cat in dashboard.categories() {
        output.push_str(&format!(
            "• {} - {} settings ({})\n",
            cat.category, cat.settings_count, cat.health
        ));
    }

    output
}

/// Check if query is about dashboard
pub fn is_dashboard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("dashboard")
        || lower.contains("settings overview")
        || lower.contains("settings summary")
        || lower.contains("settings status")
}

/// Fun fact about dashboard
pub fn settings_dashboard_fun_fact() -> &'static str {
    "Anna's dashboard gives you a bird's-eye view of all your settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_dashboard() {
        let dashboard = SettingsDashboard::new();
        let output = format_dashboard(&dashboard);
        assert!(output.contains("Dashboard"));
    }

    #[test]
    fn test_is_dashboard_query() {
        assert!(is_dashboard_query("show dashboard"));
        assert!(is_dashboard_query("settings overview"));
        assert!(!is_dashboard_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_dashboard_fun_fact();
        assert!(fact.contains("dashboard"));
    }
}
