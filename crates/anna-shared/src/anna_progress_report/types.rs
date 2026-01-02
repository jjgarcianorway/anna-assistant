//! Core data types for Anna Progress Reporting
//!
//! Defines all the fundamental types used in progress tracking including
//! time periods, trends, metrics, milestones, and snapshots.

use serde::{Deserialize, Serialize};

/// Time period for comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePeriod {
    /// Last 24 hours
    Day,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// All time
    AllTime,
}

impl TimePeriod {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Day => "Today",
            Self::Week => "This Week",
            Self::Month => "This Month",
            Self::AllTime => "All Time",
        }
    }

    /// Duration in seconds
    pub fn duration_secs(&self) -> Option<u64> {
        match self {
            Self::Day => Some(86400),
            Self::Week => Some(604800),
            Self::Month => Some(2592000),
            Self::AllTime => None,
        }
    }
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Up,
    Down,
    Stable,
}

impl Trend {
    /// Symbol for display
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Up => "[^]",
            Self::Down => "[v]",
            Self::Stable => "[-]",
        }
    }
}

/// A progress metric with trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMetric {
    /// Metric name
    pub name: String,
    /// Current value
    pub current: String,
    /// Previous value (for comparison)
    pub previous: Option<String>,
    /// Trend direction
    pub trend: Trend,
    /// Percentage change
    pub change_percent: Option<f64>,
}

impl ProgressMetric {
    /// Create a new metric
    pub fn new(name: impl Into<String>, current: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            current: current.into(),
            previous: None,
            trend: Trend::Stable,
            change_percent: None,
        }
    }

    /// Set comparison data
    pub fn with_comparison(mut self, previous: impl Into<String>, trend: Trend, change: f64) -> Self {
        self.previous = Some(previous.into());
        self.trend = trend;
        self.change_percent = Some(change);
        self
    }
}

/// Milestone achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Milestone name
    pub name: String,
    /// Description
    pub description: String,
    /// When achieved (Unix timestamp, None if not yet)
    pub achieved_at: Option<u64>,
    /// Target value
    pub target: u64,
    /// Current value
    pub current: u64,
}

impl Milestone {
    /// Create a new milestone
    pub fn new(name: impl Into<String>, description: impl Into<String>, target: u64) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            achieved_at: None,
            target,
            current: 0,
        }
    }

    /// Progress percentage
    pub fn progress_percent(&self) -> f64 {
        if self.target == 0 {
            return 100.0;
        }
        ((self.current as f64 / self.target as f64) * 100.0).min(100.0)
    }

    /// Is milestone achieved?
    pub fn is_achieved(&self) -> bool {
        self.current >= self.target
    }
}

/// Anna's progress snapshot for a period
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeriodSnapshot {
    /// Period start time
    pub start_time: u64,
    /// Period end time
    pub end_time: u64,
    /// Tickets handled
    pub tickets: u64,
    /// Tickets resolved by Anna alone
    pub anna_solo: u64,
    /// Recipes learned
    pub recipes_learned: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average resolution time ms
    pub avg_resolution_ms: u64,
}

/// Complete progress report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressReport {
    /// Report generation time
    pub generated_at: u64,
    /// Period being reported
    pub period: String,
    /// Key metrics
    pub metrics: Vec<ProgressMetric>,
    /// Milestones
    pub milestones: Vec<Milestone>,
    /// Current snapshot
    pub current: Option<PeriodSnapshot>,
    /// Previous snapshot (for comparison)
    pub previous: Option<PeriodSnapshot>,
    /// Highlights (achievements this period)
    pub highlights: Vec<String>,
    /// Areas for improvement
    pub improvements: Vec<String>,
}

impl ProgressReport {
    /// Create a new empty report
    pub fn new(period: TimePeriod) -> Self {
        Self {
            period: period.display().to_string(),
            ..Default::default()
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, metric: ProgressMetric) {
        self.metrics.push(metric);
    }

    /// Add a milestone
    pub fn add_milestone(&mut self, milestone: Milestone) {
        self.milestones.push(milestone);
    }

    /// Add a highlight
    pub fn add_highlight(&mut self, highlight: impl Into<String>) {
        self.highlights.push(highlight.into());
    }

    /// Add an improvement area
    pub fn add_improvement(&mut self, area: impl Into<String>) {
        self.improvements.push(area.into());
    }

    /// Achieved milestones
    pub fn achieved_milestones(&self) -> Vec<&Milestone> {
        self.milestones.iter().filter(|m| m.is_achieved()).collect()
    }

    /// Pending milestones
    pub fn pending_milestones(&self) -> Vec<&Milestone> {
        self.milestones.iter().filter(|m| !m.is_achieved()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_period_display() {
        assert_eq!(TimePeriod::Day.display(), "Today");
        assert_eq!(TimePeriod::Week.display(), "This Week");
        assert_eq!(TimePeriod::AllTime.duration_secs(), None);
        assert_eq!(TimePeriod::Day.duration_secs(), Some(86400));
    }

    #[test]
    fn test_trend_symbol() {
        assert_eq!(Trend::Up.symbol(), "[^]");
        assert_eq!(Trend::Down.symbol(), "[v]");
        assert_eq!(Trend::Stable.symbol(), "[-]");
    }

    #[test]
    fn test_progress_metric_new() {
        let metric = ProgressMetric::new("Tickets", "42");
        assert_eq!(metric.name, "Tickets");
        assert_eq!(metric.current, "42");
        assert_eq!(metric.trend, Trend::Stable);
    }

    #[test]
    fn test_progress_metric_with_comparison() {
        let metric = ProgressMetric::new("Tickets", "42")
            .with_comparison("30", Trend::Up, 40.0);

        assert_eq!(metric.previous, Some("30".to_string()));
        assert_eq!(metric.trend, Trend::Up);
        assert_eq!(metric.change_percent, Some(40.0));
    }

    #[test]
    fn test_milestone_progress() {
        let mut milestone = Milestone::new("Test", "Description", 100);
        milestone.current = 50;

        assert_eq!(milestone.progress_percent(), 50.0);
        assert!(!milestone.is_achieved());

        milestone.current = 100;
        assert!(milestone.is_achieved());
    }

    #[test]
    fn test_progress_report_add() {
        let mut report = ProgressReport::new(TimePeriod::Week);
        report.add_metric(ProgressMetric::new("Test", "10"));
        report.add_highlight("Did something great");
        report.add_improvement("Could do better at X");

        assert_eq!(report.metrics.len(), 1);
        assert_eq!(report.highlights.len(), 1);
        assert_eq!(report.improvements.len(), 1);
    }

    #[test]
    fn test_progress_report_milestones() {
        let mut report = ProgressReport::new(TimePeriod::Month);

        let mut achieved = Milestone::new("Done", "Done", 10);
        achieved.current = 10;
        report.add_milestone(achieved);

        let pending = Milestone::new("Todo", "Todo", 100);
        report.add_milestone(pending);

        assert_eq!(report.achieved_milestones().len(), 1);
        assert_eq!(report.pending_milestones().len(), 1);
    }
}
