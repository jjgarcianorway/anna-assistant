//! Dashboard formatting utilities.

use super::types::{DashboardSection, StatsDashboard};

/// Format dashboard for full display
pub fn format_dashboard(dashboard: &StatsDashboard) -> String {
    let mut output = String::new();

    output.push_str("Anna Statistics Dashboard\n");
    output.push_str("══════════════════════════════════════\n\n");

    // Health score
    let health_bar = generate_health_bar(dashboard.health_score);
    output.push_str(&format!("Health: {} {}%\n\n", health_bar, dashboard.health_score));

    // Sections
    for section in dashboard.active_sections() {
        output.push_str(&format!("{}:\n", section.display_name()));

        for metric in dashboard.by_section(section) {
            let trend = metric
                .trend
                .map(|t| format!(" ({})", t.symbol()))
                .unwrap_or_default();

            output.push_str(&format!("  {}: {}{}\n", metric.name, metric.value, trend));
        }
        output.push('\n');
    }

    output
}

/// Format compact dashboard
pub fn format_dashboard_compact(dashboard: &StatsDashboard) -> String {
    if dashboard.metrics.is_empty() {
        return "No stats yet".to_string();
    }

    let summary_metrics: Vec<_> = dashboard
        .by_section(DashboardSection::Summary)
        .iter()
        .map(|m| format!("{}: {}", m.name, m.value))
        .collect();

    if summary_metrics.is_empty() {
        format!(
            "Health {}%, {} metrics tracked",
            dashboard.health_score,
            dashboard.metric_count()
        )
    } else {
        format!(
            "Health {}% | {}",
            dashboard.health_score,
            summary_metrics.join(", ")
        )
    }
}

/// Generate ASCII health bar
fn generate_health_bar(score: u8) -> String {
    let filled = (score as usize) / 10;
    let empty = 10 - filled;

    let bar = format!("[{}{}]", "#".repeat(filled), "-".repeat(empty));
    bar
}

/// Format dashboard as single-line summary
pub fn format_dashboard_oneline(dashboard: &StatsDashboard) -> String {
    format!(
        "[{}%] {} metrics | {} issues",
        dashboard.health_score,
        dashboard.metric_count(),
        dashboard.active_issues
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats_dashboard::builder::DashboardBuilder;
    use crate::stats_dashboard::types::{StatMetric, StatsDashboard};

    #[test]
    fn test_health_bar() {
        assert_eq!(generate_health_bar(100), "[##########]");
        assert_eq!(generate_health_bar(50), "[#####-----]");
        assert_eq!(generate_health_bar(0), "[----------]");
    }

    #[test]
    fn test_format_compact() {
        let dashboard = DashboardBuilder::new()
            .with_summary(100, 95.0, 30)
            .with_health(90)
            .build();

        let output = format_dashboard_compact(&dashboard);
        assert!(output.contains("90%"));
    }

    #[test]
    fn test_format_oneline() {
        let mut dashboard = StatsDashboard::new();
        dashboard.set_health(85);
        dashboard.set_issues(2);
        dashboard.add_metric(StatMetric::new("Test", "1", DashboardSection::Summary));

        let output = format_dashboard_oneline(&dashboard);
        assert!(output.contains("85%"));
        assert!(output.contains("2 issues"));
    }
}
