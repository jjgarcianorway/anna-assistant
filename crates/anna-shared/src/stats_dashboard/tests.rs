//! Tests for stats dashboard.

#[cfg(test)]
mod tests {
    use crate::stats_dashboard::builder::DashboardBuilder;
    use crate::stats_dashboard::types::{DashboardSection, StatMetric, StatTrend, StatsDashboard};

    #[test]
    fn test_section_display() {
        assert_eq!(DashboardSection::Summary.display_name(), "Summary");
        assert_eq!(DashboardSection::Experts.display_name(), "Expert Performance");
    }

    #[test]
    fn test_stat_metric_new() {
        let metric = StatMetric::new("Test", "100", DashboardSection::Summary);
        assert_eq!(metric.name, "Test");
        assert_eq!(metric.value, "100");
    }

    #[test]
    fn test_metric_with_trend() {
        let metric = StatMetric::new("Test", "100", DashboardSection::Summary)
            .with_trend(StatTrend::Up);
        assert_eq!(metric.trend, Some(StatTrend::Up));
    }

    #[test]
    fn test_trend_symbol() {
        assert_eq!(StatTrend::Up.symbol(), "+");
        assert_eq!(StatTrend::Down.symbol(), "-");
        assert_eq!(StatTrend::Stable.symbol(), "=");
    }

    #[test]
    fn test_dashboard_add_metric() {
        let mut dashboard = StatsDashboard::new();

        dashboard.add_metric(StatMetric::new("Test", "100", DashboardSection::Summary));

        assert_eq!(dashboard.metric_count(), 1);
    }

    #[test]
    fn test_dashboard_by_section() {
        let mut dashboard = StatsDashboard::new();

        dashboard.add_metric(StatMetric::new("S1", "1", DashboardSection::Summary));
        dashboard.add_metric(StatMetric::new("S2", "2", DashboardSection::Summary));
        dashboard.add_metric(StatMetric::new("R1", "3", DashboardSection::Resolutions));

        let summary = dashboard.by_section(DashboardSection::Summary);
        assert_eq!(summary.len(), 2);
    }

    #[test]
    fn test_dashboard_builder() {
        let dashboard = DashboardBuilder::new()
            .with_summary(100, 95.5, 30)
            .with_health(85)
            .build();

        assert_eq!(dashboard.health_score, 85);
        assert_eq!(dashboard.by_section(DashboardSection::Summary).len(), 3);
    }

    #[test]
    fn test_builder_all_sections() {
        let dashboard = DashboardBuilder::new()
            .with_summary(100, 95.0, 30)
            .with_resolutions(5000.0, 500, 30000)
            .with_interactions(500, 3.5, 40.0)
            .with_experts(5, 30, 20)
            .with_recipes(50, 25, 200)
            .with_responses(100, 500.0, 80.0)
            .build();

        assert!(dashboard.metric_count() >= 15);
    }

    #[test]
    fn test_active_sections() {
        let mut dashboard = StatsDashboard::new();

        dashboard.add_metric(StatMetric::new("S1", "1", DashboardSection::Summary));
        dashboard.add_metric(StatMetric::new("R1", "2", DashboardSection::Recipes));

        let sections = dashboard.active_sections();
        assert_eq!(sections.len(), 2);
    }
}
