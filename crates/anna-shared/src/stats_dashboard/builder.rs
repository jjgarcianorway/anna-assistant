//! Dashboard builder for easy construction.

use super::types::{DashboardSection, StatMetric, StatsDashboard};

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
