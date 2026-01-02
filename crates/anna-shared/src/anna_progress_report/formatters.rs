//! Formatting functions for progress reports
//!
//! Provides various output formats for progress reports including
//! full display, compact, one-line, and progress bars.

use super::types::{ProgressReport, Trend};

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
    use crate::anna_progress_report::types::{ProgressMetric, TimePeriod};

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
    fn test_progress_summary_message() {
        let report = ProgressReport::new(TimePeriod::Week);
        let msg = progress_summary_message(&report);
        assert!(msg.contains("Steady progress"));
    }
}
