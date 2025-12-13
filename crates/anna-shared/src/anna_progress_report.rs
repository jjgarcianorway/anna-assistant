//! Anna Progress Report (Phase 72)
//!
//! Generates comprehensive progress reports showing Anna's learning journey,
//! improvements over time, and achievements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Calculate trend from two values
pub fn calculate_trend(current: f64, previous: f64) -> Trend {
    let change = (current - previous).abs();
    let threshold = previous.abs() * 0.05; // 5% threshold

    if current > previous + threshold {
        Trend::Up
    } else if current < previous - threshold {
        Trend::Down
    } else {
        Trend::Stable
    }
}

/// Calculate percentage change
pub fn calculate_change_percent(current: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        if current > 0.0 {
            return 100.0;
        }
        return 0.0;
    }
    ((current - previous) / previous) * 100.0
}

/// Generate progress bar
pub fn progress_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}] {:.0}%", "=".repeat(filled), " ".repeat(empty), percent)
}

/// Format progress report as full display
pub fn format_progress_report(report: &ProgressReport) -> String {
    let mut lines = Vec::new();

    lines.push(format!("=== Anna Progress Report: {} ===", report.period));
    lines.push(String::new());

    // Key metrics
    if !report.metrics.is_empty() {
        lines.push("--- Key Metrics ---".to_string());
        for metric in &report.metrics {
            let trend = metric.trend.symbol();
            let change = metric
                .change_percent
                .map(|c| format!(" ({:+.1}%)", c))
                .unwrap_or_default();
            lines.push(format!("{} {}: {}{}", trend, metric.name, metric.current, change));
        }
        lines.push(String::new());
    }

    // Highlights
    if !report.highlights.is_empty() {
        lines.push("--- Highlights ---".to_string());
        for highlight in &report.highlights {
            lines.push(format!("  * {}", highlight));
        }
        lines.push(String::new());
    }

    // Milestones
    let achieved = report.achieved_milestones();
    if !achieved.is_empty() {
        lines.push("--- Achievements Unlocked ---".to_string());
        for milestone in achieved {
            lines.push(format!("  [*] {}", milestone.name));
        }
        lines.push(String::new());
    }

    let pending = report.pending_milestones();
    if !pending.is_empty() {
        lines.push("--- Next Milestones ---".to_string());
        for milestone in pending.iter().take(3) {
            let bar = progress_bar(milestone.progress_percent(), 15);
            lines.push(format!("  {} {}", milestone.name, bar));
        }
        lines.push(String::new());
    }

    // Areas for improvement
    if !report.improvements.is_empty() {
        lines.push("--- Areas for Growth ---".to_string());
        for area in &report.improvements {
            lines.push(format!("  - {}", area));
        }
    }

    lines.join("\n")
}

/// Format progress report compact
pub fn format_progress_report_compact(report: &ProgressReport) -> String {
    let mut parts = Vec::new();

    // Top 3 metrics
    for metric in report.metrics.iter().take(3) {
        let trend = metric.trend.symbol();
        parts.push(format!("{}: {}{}", metric.name, metric.current, trend));
    }

    if parts.is_empty() {
        return "No progress data available.".to_string();
    }

    parts.join(" | ")
}

/// Format progress report one-line
pub fn format_progress_report_oneline(report: &ProgressReport) -> String {
    let achieved = report.achieved_milestones().len();
    let pending = report.pending_milestones().len();

    format!(
        "{}: {} achievements, {} in progress, {} metrics tracked",
        report.period,
        achieved,
        pending,
        report.metrics.len()
    )
}

/// Generate default milestones for Anna
pub fn default_milestones() -> Vec<Milestone> {
    vec![
        Milestone::new("First Ticket", "Handle your first support ticket", 1),
        Milestone::new("Getting Started", "Handle 10 tickets", 10),
        Milestone::new("Finding Rhythm", "Handle 50 tickets", 50),
        Milestone::new("Century", "Handle 100 tickets", 100),
        Milestone::new("Seasoned Pro", "Handle 500 tickets", 500),
        Milestone::new("First Recipe", "Learn your first recipe", 1),
        Milestone::new("Recipe Collection", "Learn 10 recipes", 10),
        Milestone::new("Recipe Master", "Learn 50 recipes", 50),
        Milestone::new("Independence Day", "Solve 10 tickets without help", 10),
        Milestone::new("Solo Artist", "Solve 50 tickets without help", 50),
    ]
}

/// Check if query is asking about progress
pub fn is_progress_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "progress report",
        "my progress",
        "anna progress",
        "how am i doing",
        "how are we doing",
        "learning progress",
        "show progress",
        "what have you learned",
        "improvements",
        "milestones",
        "achievements",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Generate a progress summary message
pub fn progress_summary_message(report: &ProgressReport) -> String {
    let achieved = report.achieved_milestones().len();
    let highlights = report.highlights.len();

    if achieved > 0 && highlights > 0 {
        return format!(
            "Great progress! {} achievements and {} highlights this {}.",
            achieved,
            highlights,
            report.period.to_lowercase()
        );
    }

    if achieved > 0 {
        return format!(
            "{} achievement{} unlocked this {}!",
            achieved,
            if achieved == 1 { "" } else { "s" },
            report.period.to_lowercase()
        );
    }

    if !report.metrics.is_empty() {
        let up_count = report.metrics.iter().filter(|m| m.trend == Trend::Up).count();
        if up_count > 0 {
            return format!(
                "{} metric{} improved this {}.",
                up_count,
                if up_count == 1 { "" } else { "s" },
                report.period.to_lowercase()
            );
        }
    }

    format!("Steady progress this {}.", report.period.to_lowercase())
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

    #[test]
    fn test_calculate_trend() {
        assert_eq!(calculate_trend(100.0, 80.0), Trend::Up);
        assert_eq!(calculate_trend(80.0, 100.0), Trend::Down);
        assert_eq!(calculate_trend(100.0, 99.0), Trend::Stable);
    }

    #[test]
    fn test_calculate_change_percent() {
        assert!((calculate_change_percent(150.0, 100.0) - 50.0).abs() < 0.1);
        assert!((calculate_change_percent(50.0, 100.0) - (-50.0)).abs() < 0.1);
        assert_eq!(calculate_change_percent(100.0, 0.0), 100.0);
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(50.0, 10), "[=====     ] 50%");
        assert_eq!(progress_bar(100.0, 10), "[==========] 100%");
        assert_eq!(progress_bar(0.0, 10), "[          ] 0%");
    }

    #[test]
    fn test_format_progress_report() {
        let mut report = ProgressReport::new(TimePeriod::Week);
        report.add_metric(ProgressMetric::new("Tickets", "42"));
        report.add_highlight("Learned 5 new recipes");

        let output = format_progress_report(&report);
        assert!(output.contains("Progress Report"));
        assert!(output.contains("Tickets: 42"));
        assert!(output.contains("Learned 5 new recipes"));
    }

    #[test]
    fn test_default_milestones() {
        let milestones = default_milestones();
        assert!(!milestones.is_empty());
        assert!(milestones.iter().any(|m| m.name == "First Ticket"));
    }

    #[test]
    fn test_is_progress_query() {
        assert!(is_progress_query("show me my progress report"));
        assert!(is_progress_query("how am i doing?"));
        assert!(is_progress_query("what have you learned so far?"));
        assert!(is_progress_query("show milestones"));
        assert!(!is_progress_query("how do I install vim?"));
    }

    #[test]
    fn test_progress_summary_message() {
        let report = ProgressReport::new(TimePeriod::Week);
        let msg = progress_summary_message(&report);
        assert!(msg.contains("Steady progress"));
    }
}
