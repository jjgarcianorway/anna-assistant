//! Aggregated Stats Dashboard (v0.0.491).
//!
//! Provides a unified view of all statistics.
//! Combines data from multiple stats modules into one dashboard.

use serde::{Deserialize, Serialize};

/// Dashboard section type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardSection {
    /// Overall summary
    Summary,
    /// Quick status
    Status,
    /// Resolution times
    Resolutions,
    /// Interactions
    Interactions,
    /// Expert performance
    Experts,
    /// Recipe statistics
    Recipes,
    /// Response lengths
    Responses,
    /// Repeated questions
    Questions,
}

impl DashboardSection {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Status => "System Status",
            Self::Resolutions => "Resolution Times",
            Self::Interactions => "Interactions",
            Self::Experts => "Expert Performance",
            Self::Recipes => "Recipes",
            Self::Responses => "Response Lengths",
            Self::Questions => "Questions",
        }
    }

    /// All sections
    pub fn all() -> Vec<Self> {
        vec![
            Self::Summary,
            Self::Status,
            Self::Resolutions,
            Self::Interactions,
            Self::Experts,
            Self::Recipes,
            Self::Responses,
            Self::Questions,
        ]
    }
}

/// A single stat metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatMetric {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: String,
    /// Optional trend (up, down, stable)
    pub trend: Option<StatTrend>,
    /// Section this belongs to
    pub section: DashboardSection,
}

impl StatMetric {
    /// Create new metric
    pub fn new(name: &str, value: &str, section: DashboardSection) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            trend: None,
            section,
        }
    }

    /// Add trend
    pub fn with_trend(mut self, trend: StatTrend) -> Self {
        self.trend = Some(trend);
        self
    }
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatTrend {
    /// Improving
    Up,
    /// Declining
    Down,
    /// No change
    Stable,
}

impl StatTrend {
    /// Get symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Up => "+",
            Self::Down => "-",
            Self::Stable => "=",
        }
    }
}

/// Dashboard data container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatsDashboard {
    /// All metrics
    pub metrics: Vec<StatMetric>,
    /// Overall health score (0-100)
    pub health_score: u8,
    /// Last updated timestamp
    pub last_updated: u64,
    /// Active issues count
    pub active_issues: u32,
}

impl StatsDashboard {
    /// Create new dashboard
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a metric
    pub fn add_metric(&mut self, metric: StatMetric) {
        self.metrics.push(metric);
    }

    /// Set health score
    pub fn set_health(&mut self, score: u8) {
        self.health_score = score.min(100);
    }

    /// Set last updated
    pub fn set_updated(&mut self, timestamp: u64) {
        self.last_updated = timestamp;
    }

    /// Set active issues
    pub fn set_issues(&mut self, count: u32) {
        self.active_issues = count;
    }

    /// Get metrics by section
    pub fn by_section(&self, section: DashboardSection) -> Vec<&StatMetric> {
        self.metrics.iter().filter(|m| m.section == section).collect()
    }

    /// Get all section names with metrics
    pub fn active_sections(&self) -> Vec<DashboardSection> {
        let mut sections: Vec<DashboardSection> = self
            .metrics
            .iter()
            .map(|m| m.section)
            .collect();
        sections.sort_by_key(|s| *s as u8);
        sections.dedup();
        sections
    }

    /// Count metrics
    pub fn metric_count(&self) -> usize {
        self.metrics.len()
    }
}

/// Dashboard builder for easy construction
#[derive(Debug, Clone, Default)]
pub struct DashboardBuilder {
    dashboard: StatsDashboard,
}

impl DashboardBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add summary metrics
    pub fn with_summary(mut self, total_tickets: u64, success_rate: f64, uptime_days: u64) -> Self {
        self.dashboard.add_metric(StatMetric::new(
            "Total Tickets",
            &total_tickets.to_string(),
            DashboardSection::Summary,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Success Rate",
            &format!("{:.1}%", success_rate),
            DashboardSection::Summary,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Uptime",
            &format!("{} days", uptime_days),
            DashboardSection::Summary,
        ));
        self
    }

    /// Add resolution metrics
    pub fn with_resolutions(mut self, avg_ms: f64, fastest_ms: u64, slowest_ms: u64) -> Self {
        self.dashboard.add_metric(StatMetric::new(
            "Avg Resolution",
            &format_duration(avg_ms as u64),
            DashboardSection::Resolutions,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Fastest",
            &format_duration(fastest_ms),
            DashboardSection::Resolutions,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Slowest",
            &format_duration(slowest_ms),
            DashboardSection::Resolutions,
        ));
        self
    }

    /// Add interaction metrics
    pub fn with_interactions(mut self, total: u64, avg_per_ticket: f64, anna_solo_rate: f64) -> Self {
        self.dashboard.add_metric(StatMetric::new(
            "Total Interactions",
            &total.to_string(),
            DashboardSection::Interactions,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Avg per Ticket",
            &format!("{:.1}", avg_per_ticket),
            DashboardSection::Interactions,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Anna Solo",
            &format!("{:.0}%", anna_solo_rate),
            DashboardSection::Interactions,
        ));
        self
    }

    /// Add expert metrics
    pub fn with_experts(mut self, total_experts: usize, jr_tickets: u64, sr_tickets: u64) -> Self {
        self.dashboard.add_metric(StatMetric::new(
            "Active Experts",
            &total_experts.to_string(),
            DashboardSection::Experts,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Junior Tickets",
            &jr_tickets.to_string(),
            DashboardSection::Experts,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Senior Tickets",
            &sr_tickets.to_string(),
            DashboardSection::Experts,
        ));
        self
    }

    /// Add recipe metrics
    pub fn with_recipes(mut self, total: usize, learned: u64, uses: u64) -> Self {
        self.dashboard.add_metric(StatMetric::new(
            "Total Recipes",
            &total.to_string(),
            DashboardSection::Recipes,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Learned",
            &learned.to_string(),
            DashboardSection::Recipes,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Total Uses",
            &uses.to_string(),
            DashboardSection::Recipes,
        ));
        self
    }

    /// Add response metrics
    pub fn with_responses(mut self, total: u64, avg_chars: f64, avg_words: f64) -> Self {
        self.dashboard.add_metric(StatMetric::new(
            "Total Responses",
            &total.to_string(),
            DashboardSection::Responses,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Avg Chars",
            &format!("{:.0}", avg_chars),
            DashboardSection::Responses,
        ));
        self.dashboard.add_metric(StatMetric::new(
            "Avg Words",
            &format!("{:.0}", avg_words),
            DashboardSection::Responses,
        ));
        self
    }

    /// Set health score
    pub fn with_health(mut self, score: u8) -> Self {
        self.dashboard.set_health(score);
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.dashboard.set_updated(timestamp);
        self
    }

    /// Build the dashboard
    pub fn build(self) -> StatsDashboard {
        self.dashboard
    }
}

/// Format duration in ms to human readable
fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

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

/// Check if query is asking for dashboard
pub fn is_dashboard_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "dashboard",
        "all stats",
        "full stats",
        "overview",
        "summary stats",
        "stat summary",
        "show stats",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

/// Check which section is being requested
pub fn detect_section(query: &str) -> Option<DashboardSection> {
    let lower = query.to_lowercase();

    if lower.contains("resolution") || lower.contains("time") {
        Some(DashboardSection::Resolutions)
    } else if lower.contains("interaction") || lower.contains("communication") {
        Some(DashboardSection::Interactions)
    } else if lower.contains("expert") || lower.contains("specialist") {
        Some(DashboardSection::Experts)
    } else if lower.contains("recipe") {
        Some(DashboardSection::Recipes)
    } else if lower.contains("response") || lower.contains("length") {
        Some(DashboardSection::Responses)
    } else if lower.contains("question") || lower.contains("repeated") {
        Some(DashboardSection::Questions)
    } else if lower.contains("status") || lower.contains("health") {
        Some(DashboardSection::Status)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_health_bar() {
        assert_eq!(generate_health_bar(100), "[##########]");
        assert_eq!(generate_health_bar(50), "[#####-----]");
        assert_eq!(generate_health_bar(0), "[----------]");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(2500), "2.5s");
        assert!(format_duration(65000).contains("m"));
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

    #[test]
    fn test_is_dashboard_query() {
        assert!(is_dashboard_query("show dashboard"));
        assert!(is_dashboard_query("all stats"));
        assert!(is_dashboard_query("overview"));

        assert!(!is_dashboard_query("install vim"));
    }

    #[test]
    fn test_detect_section() {
        assert_eq!(detect_section("resolution times"), Some(DashboardSection::Resolutions));
        assert_eq!(detect_section("expert stats"), Some(DashboardSection::Experts));
        assert_eq!(detect_section("recipe info"), Some(DashboardSection::Recipes));
        assert_eq!(detect_section("random query"), None);
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
