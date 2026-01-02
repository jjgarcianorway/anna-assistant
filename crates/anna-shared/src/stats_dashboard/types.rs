//! Core types for stats dashboard.

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
