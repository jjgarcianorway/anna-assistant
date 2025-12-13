//! Anna Metrics Dashboard - Phase 100 🎉
//!
//! Comprehensive dashboard combining all Anna metrics.
//! The 100th feature milestone!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dashboard section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DashboardSection {
    #[default]
    Overview,
    Tickets,
    Recipes,
    Learning,
    Hardware,
    Performance,
    Sessions,
    Errors,
}

impl DashboardSection {
    pub fn name(&self) -> &'static str {
        match self {
            DashboardSection::Overview => "Overview",
            DashboardSection::Tickets => "Tickets",
            DashboardSection::Recipes => "Recipes",
            DashboardSection::Learning => "Learning",
            DashboardSection::Hardware => "Hardware",
            DashboardSection::Performance => "Performance",
            DashboardSection::Sessions => "Sessions",
            DashboardSection::Errors => "Errors",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            DashboardSection::Overview => "◉",
            DashboardSection::Tickets => "🎫",
            DashboardSection::Recipes => "📋",
            DashboardSection::Learning => "📚",
            DashboardSection::Hardware => "⚙",
            DashboardSection::Performance => "📊",
            DashboardSection::Sessions => "👤",
            DashboardSection::Errors => "⚠",
        }
    }
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HealthStatus {
    #[default]
    Healthy,
    Warning,
    Critical,
    Unknown,
}

impl HealthStatus {
    pub fn name(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "Healthy",
            HealthStatus::Warning => "Warning",
            HealthStatus::Critical => "Critical",
            HealthStatus::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "✓",
            HealthStatus::Warning => "!",
            HealthStatus::Critical => "✗",
            HealthStatus::Unknown => "?",
        }
    }
}

/// A metric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEntry {
    /// Metric name
    pub name: String,
    /// Current value
    pub value: String,
    /// Section
    pub section: DashboardSection,
    /// Trend (positive = good)
    pub trend: i8,
    /// Last updated
    pub updated_at: u64,
}

/// Anna metrics dashboard
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnaMetricsDashboard {
    /// All metrics
    pub metrics: Vec<MetricEntry>,
    /// Count by section
    pub by_section: HashMap<String, u64>,
    /// Overall health
    pub health: HealthStatus,
    /// Last refresh
    pub last_refresh: Option<u64>,
    /// Version milestone
    pub version: String,
}

impl AnnaMetricsDashboard {
    pub fn new() -> Self {
        Self {
            version: "0.0.524".to_string(),
            ..Default::default()
        }
    }

    /// Add or update a metric
    pub fn set_metric(&mut self, name: &str, value: &str, section: DashboardSection, timestamp: u64) {
        let found = self.metrics.iter().position(|m| m.name == name);
        if let Some(idx) = found {
            self.metrics[idx].value = value.to_string();
            self.metrics[idx].updated_at = timestamp;
        } else {
            let metric = MetricEntry {
                name: name.to_string(),
                value: value.to_string(),
                section,
                trend: 0,
                updated_at: timestamp,
            };
            *self.by_section.entry(section.name().to_string()).or_insert(0) += 1;
            self.metrics.push(metric);
        }
    }

    /// Set metric with trend
    pub fn set_metric_with_trend(
        &mut self,
        name: &str,
        value: &str,
        section: DashboardSection,
        trend: i8,
        timestamp: u64,
    ) {
        self.set_metric(name, value, section, timestamp);
        if let Some(m) = self.metrics.iter_mut().find(|m| m.name == name) {
            m.trend = trend;
        }
    }

    /// Get metric by name
    pub fn get(&self, name: &str) -> Option<&MetricEntry> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Get metrics by section
    pub fn by_dash_section(&self, section: DashboardSection) -> Vec<&MetricEntry> {
        self.metrics.iter().filter(|m| m.section == section).collect()
    }

    /// Update overall health
    pub fn update_health(&mut self, health: HealthStatus) {
        self.health = health;
    }

    /// Refresh timestamp
    pub fn refresh(&mut self, timestamp: u64) {
        self.last_refresh = Some(timestamp);
    }

    /// Total metric count
    pub fn total_count(&self) -> usize {
        self.metrics.len()
    }

    /// Get positive trends count
    pub fn positive_trends(&self) -> usize {
        self.metrics.iter().filter(|m| m.trend > 0).count()
    }

    /// Get negative trends count
    pub fn negative_trends(&self) -> usize {
        self.metrics.iter().filter(|m| m.trend < 0).count()
    }
}

/// Format dashboard for display
pub fn format_dashboard(dashboard: &AnnaMetricsDashboard) -> String {
    let mut lines = vec!["╔══════════════════════════════════════╗".to_string()];
    lines.push("║    Anna Metrics Dashboard v0.0.524   ║".to_string());
    lines.push("║      🎉 Phase 100 Milestone! 🎉      ║".to_string());
    lines.push("╚══════════════════════════════════════╝".to_string());
    lines.push(String::new());

    // Health
    lines.push(format!(
        "Overall Health: [{}] {}",
        dashboard.health.symbol(),
        dashboard.health.name()
    ));
    lines.push(format!("Total Metrics: {}", dashboard.total_count()));
    lines.push(format!(
        "Trends: {} positive, {} negative",
        dashboard.positive_trends(),
        dashboard.negative_trends()
    ));

    // By section
    if !dashboard.by_section.is_empty() {
        lines.push(String::new());
        lines.push("Metrics by section:".to_string());
        for (section, count) in &dashboard.by_section {
            lines.push(format!("  {}: {}", section, count));
        }
    }

    // Overview metrics
    let overview = dashboard.by_dash_section(DashboardSection::Overview);
    if !overview.is_empty() {
        lines.push(String::new());
        lines.push("Overview:".to_string());
        for metric in overview.iter().take(10) {
            let trend_sym = match metric.trend.cmp(&0) {
                std::cmp::Ordering::Greater => "↑",
                std::cmp::Ordering::Less => "↓",
                std::cmp::Ordering::Equal => "→",
            };
            lines.push(format!("  {} {} = {}", trend_sym, metric.name, metric.value));
        }
    }

    lines.join("\n")
}

/// Format dashboard compact
pub fn format_dashboard_compact(dashboard: &AnnaMetricsDashboard) -> String {
    format!(
        "Dashboard: {} metrics | {} sections | Health: {}",
        dashboard.total_count(),
        dashboard.by_section.len(),
        dashboard.health.name()
    )
}

/// Format dashboard one-line
pub fn format_dashboard_oneline(dashboard: &AnnaMetricsDashboard) -> String {
    format!(
        "[{}] {} metrics",
        dashboard.health.symbol(),
        dashboard.total_count()
    )
}

/// Check if query is about dashboard
pub fn is_dashboard_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "dashboard",
        "metrics",
        "overview",
        "all stats",
        "health status",
        "anna status",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about dashboard
pub fn dashboard_fun_fact(dashboard: &AnnaMetricsDashboard) -> String {
    if dashboard.metrics.is_empty() {
        return "Dashboard is ready for Phase 100! 🎉".to_string();
    }

    let facts = [
        format!("Anna tracks {} metrics across all systems.", dashboard.total_count()),
        format!("{} metrics show positive trends! 📈", dashboard.positive_trends()),
        "Phase 100 milestone reached! 🎉".to_string(),
    ];

    facts[dashboard.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_section() {
        assert_eq!(DashboardSection::Overview.name(), "Overview");
        assert_eq!(DashboardSection::Tickets.symbol(), "🎫");
    }

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy.name(), "Healthy");
        assert_eq!(HealthStatus::Critical.symbol(), "✗");
    }

    #[test]
    fn test_set_metric() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.set_metric("tickets_resolved", "100", DashboardSection::Tickets, 1000);

        assert_eq!(dashboard.total_count(), 1);
        assert!(dashboard.get("tickets_resolved").is_some());
    }

    #[test]
    fn test_update_metric() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.set_metric("tickets_resolved", "100", DashboardSection::Tickets, 1000);
        dashboard.set_metric("tickets_resolved", "150", DashboardSection::Tickets, 2000);

        assert_eq!(dashboard.total_count(), 1);
        assert_eq!(dashboard.get("tickets_resolved").unwrap().value, "150");
    }

    #[test]
    fn test_set_with_trend() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.set_metric_with_trend("success_rate", "95%", DashboardSection::Performance, 1, 1000);

        let metric = dashboard.get("success_rate").unwrap();
        assert_eq!(metric.trend, 1);
    }

    #[test]
    fn test_by_section() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.set_metric("m1", "v1", DashboardSection::Tickets, 1000);
        dashboard.set_metric("m2", "v2", DashboardSection::Recipes, 1000);

        assert_eq!(dashboard.by_dash_section(DashboardSection::Tickets).len(), 1);
        assert_eq!(dashboard.by_dash_section(DashboardSection::Recipes).len(), 1);
    }

    #[test]
    fn test_trends() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.set_metric_with_trend("m1", "v1", DashboardSection::Overview, 1, 1000);
        dashboard.set_metric_with_trend("m2", "v2", DashboardSection::Overview, -1, 1000);
        dashboard.set_metric_with_trend("m3", "v3", DashboardSection::Overview, 0, 1000);

        assert_eq!(dashboard.positive_trends(), 1);
        assert_eq!(dashboard.negative_trends(), 1);
    }

    #[test]
    fn test_health() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.update_health(HealthStatus::Warning);

        assert_eq!(dashboard.health, HealthStatus::Warning);
    }

    #[test]
    fn test_format_dashboard() {
        let mut dashboard = AnnaMetricsDashboard::new();
        dashboard.set_metric("test", "value", DashboardSection::Overview, 1000);

        let output = format_dashboard(&dashboard);
        assert!(output.contains("Anna Metrics Dashboard"));
        assert!(output.contains("Phase 100"));
    }

    #[test]
    fn test_is_dashboard_query() {
        assert!(is_dashboard_query("show dashboard"));
        assert!(is_dashboard_query("anna status"));
        assert!(is_dashboard_query("all stats"));
        assert!(!is_dashboard_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let dashboard = AnnaMetricsDashboard::new();
        let fact = dashboard_fun_fact(&dashboard);
        assert!(fact.contains("Phase 100"));
    }
}
