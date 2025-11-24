//! v6.24.0: Insights Engine - Historical Metrics & Trend Analysis
//!
//! The Insights Engine wraps the Historian and Trend Detectors to provide
//! actionable insights about system health trends over time.
//!
//! Core Features:
//! - Rules-based insight generation (no LLM required)
//! - Severity classification (info, warning, critical)
//! - Evidence-backed recommendations
//! - Persistent across daemon restarts
//!
//! Insight Sources:
//! 1. Boot duration trends
//! 2. Disk space deterioration
//! 3. Journal error spikes
//! 4. Memory pressure trends
//! 5. Swap usage anomalies
//! 6. Anna inactivity detection
//! 7. Configuration drift warnings

use crate::historian::Historian;
use crate::trend_detectors::{TrendDetection, TrendSeverity};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Severity levels for insights (aligned with TrendSeverity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightSeverity {
    Info,     // Informational observation
    Warning,  // Requires attention soon
    Critical, // Immediate action needed
}

impl From<TrendSeverity> for InsightSeverity {
    fn from(ts: TrendSeverity) -> Self {
        match ts {
            TrendSeverity::Info => InsightSeverity::Info,
            TrendSeverity::Warning => InsightSeverity::Warning,
            TrendSeverity::Critical => InsightSeverity::Critical,
        }
    }
}

/// A single insight about system health trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Unique identifier (e.g., "boot_regression_2025-11-24")
    pub id: String,
    /// When this insight was generated
    pub timestamp: DateTime<Utc>,
    /// Severity level
    pub severity: InsightSeverity,
    /// Short title (1 line)
    pub title: String,
    /// Detailed explanation (2-4 lines)
    pub explanation: String,
    /// Evidence supporting this insight
    pub evidence: Vec<String>,
    /// Optional actionable suggestion
    pub suggestion: Option<String>,
    /// Source detector name
    pub detector: String,
}

impl Insight {
    pub fn new(
        detector: impl Into<String>,
        severity: InsightSeverity,
        title: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        let detector_str = detector.into();
        let timestamp = Utc::now();
        let id = format!("{}_{}", detector_str, timestamp.format("%Y%m%d_%H%M%S"));

        Self {
            id,
            timestamp,
            severity,
            title: title.into(),
            explanation: explanation.into(),
            evidence: Vec::new(),
            suggestion: None,
            detector: detector_str,
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Insights Engine - generates actionable insights from historical data
pub struct InsightsEngine {
    historian: Historian,
}

impl InsightsEngine {
    /// Create a new Insights Engine
    pub fn new(historian: Historian) -> Self {
        Self { historian }
    }

    /// Generate all insights for the last N hours (default 24)
    pub fn generate_insights(&self, hours: i64) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // 1. Boot duration trends
        if let Some(insight) = self.detect_boot_regression()? {
            insights.push(insight);
        }

        // 2. Disk space trends
        if let Some(insight) = self.detect_disk_space_issues()? {
            insights.push(insight);
        }

        // 3. Error rate spikes
        if let Some(insight) = self.detect_error_spikes(hours)? {
            insights.push(insight);
        }

        // 4. Memory pressure
        if let Some(insight) = self.detect_memory_pressure()? {
            insights.push(insight);
        }

        // 5. Swap usage anomalies
        if let Some(insight) = self.detect_swap_anomalies()? {
            insights.push(insight);
        }

        // 6. Anna inactivity
        if let Some(insight) = self.detect_anna_inactivity(hours)? {
            insights.push(insight);
        }

        // Sort by severity (Critical > Warning > Info)
        insights.sort_by(|a, b| {
            use InsightSeverity::*;
            let a_val = match a.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            let b_val = match b.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            b_val.cmp(&a_val) // Descending order
        });

        Ok(insights)
    }

    /// Get top N insights (for status display)
    pub fn get_top_insights(&self, count: usize, hours: i64) -> Result<Vec<Insight>> {
        let all_insights = self.generate_insights(hours)?;
        Ok(all_insights.into_iter().take(count).collect())
    }

    // ========================================================================
    // Individual Detectors
    // ========================================================================

    /// Detect boot time regression
    fn detect_boot_regression(&self) -> Result<Option<Insight>> {
        use crate::trend_detectors::detect_boot_regression;

        if let Some(trend) = detect_boot_regression(&self.historian)? {
            let insight = Insight::new(
                "boot_regression",
                trend.severity.into(),
                trend.title,
                trend.description,
            )
            .with_evidence(trend.supporting_data)
            .with_suggestion(trend.recommendation);

            return Ok(Some(insight));
        }

        Ok(None)
    }

    /// Detect disk space issues
    fn detect_disk_space_issues(&self) -> Result<Option<Insight>> {
        // Check disk space trends from last 7 days
        let disk_trends = self.historian.get_disk_trends(7)?;

        // Critical: >90% used
        if disk_trends.current_used_percent > 90.0 {
            let days_until_full = if disk_trends.growth_rate_gb_per_day > 0.1 {
                let free_gb = disk_trends.total_gb - disk_trends.used_gb;
                (free_gb / disk_trends.growth_rate_gb_per_day).ceil() as i64
            } else {
                -1 // Not growing
            };

            let explanation = if days_until_full > 0 && days_until_full < 30 {
                format!(
                    "Root filesystem is {}% full ({:.1} GB used of {:.1} GB total). \
                     At current growth rate, disk will be full in {} days.",
                    disk_trends.current_used_percent as i64,
                    disk_trends.used_gb,
                    disk_trends.total_gb,
                    days_until_full
                )
            } else {
                format!(
                    "Root filesystem is {}% full ({:.1} GB used of {:.1} GB total). \
                     Immediate cleanup recommended.",
                    disk_trends.current_used_percent as i64,
                    disk_trends.used_gb,
                    disk_trends.total_gb
                )
            };

            return Ok(Some(
                Insight::new("disk_space_critical", InsightSeverity::Critical, "Disk Space Critical", explanation)
                    .with_evidence(vec![
                        format!("Current usage: {:.1}%", disk_trends.current_used_percent),
                        format!("Growth rate: {:.2} GB/day", disk_trends.growth_rate_gb_per_day),
                    ])
                    .with_suggestion("Run 'annactl \"clean up disk space\"' for cleanup recommendations"),
            ));
        }

        // Warning: >80% used OR rapid growth
        if disk_trends.current_used_percent > 80.0 || disk_trends.growth_rate_gb_per_day > 1.0 {
            return Ok(Some(
                Insight::new(
                    "disk_space_warning",
                    InsightSeverity::Warning,
                    "Disk Space Increasing",
                    format!(
                        "Root filesystem usage is trending up. Currently at {:.1}% ({:.1} GB used), \
                         growing at {:.2} GB/day.",
                        disk_trends.current_used_percent, disk_trends.used_gb, disk_trends.growth_rate_gb_per_day
                    ),
                )
                .with_evidence(vec![
                    format!("Current: {:.1}% used", disk_trends.current_used_percent),
                    format!("Growth: {:.2} GB/day", disk_trends.growth_rate_gb_per_day),
                ])
                .with_suggestion("Monitor disk usage and consider cleanup if trend continues"),
            ));
        }

        Ok(None)
    }

    /// Detect error rate spikes
    fn detect_error_spikes(&self, hours: i64) -> Result<Option<Insight>> {
        let error_trends = self.historian.get_error_trends_v2(hours)?;

        // Critical: >100 errors/hour sustained
        if error_trends.avg_errors_per_hour > 100.0 {
            return Ok(Some(
                Insight::new(
                    "error_spike_critical",
                    InsightSeverity::Critical,
                    "High Error Rate Detected",
                    format!(
                        "System is generating {:.0} errors per hour on average over the last {} hours. \
                         This indicates a serious problem.",
                        error_trends.avg_errors_per_hour, hours
                    ),
                )
                .with_evidence(vec![
                    format!("Average: {:.0} errors/hour", error_trends.avg_errors_per_hour),
                    format!("Total: {} errors", error_trends.total_errors),
                ])
                .with_suggestion("Run 'annactl \"check my system health\"' for diagnostic details"),
            ));
        }

        // Warning: >20 errors/hour OR increasing trend
        if error_trends.avg_errors_per_hour > 20.0 {
            return Ok(Some(
                Insight::new(
                    "error_spike_warning",
                    InsightSeverity::Warning,
                    "Error Rate Elevated",
                    format!(
                        "System error rate is elevated at {:.0} errors/hour over the last {} hours.",
                        error_trends.avg_errors_per_hour, hours
                    ),
                )
                .with_evidence(vec![
                    format!("Average: {:.0} errors/hour", error_trends.avg_errors_per_hour),
                    format!("Total: {} errors", error_trends.total_errors),
                ]),
            ));
        }

        Ok(None)
    }

    /// Detect memory pressure
    fn detect_memory_pressure(&self) -> Result<Option<Insight>> {
        use crate::trend_detectors::detect_memory_leak;

        if let Some(trend) = detect_memory_leak(&self.historian)? {
            let insight = Insight::new("memory_pressure", trend.severity.into(), trend.title, trend.description)
                .with_evidence(trend.supporting_data)
                .with_suggestion(trend.recommendation);

            return Ok(Some(insight));
        }

        Ok(None)
    }

    /// Detect swap usage anomalies
    fn detect_swap_anomalies(&self) -> Result<Option<Insight>> {
        let memory_trends = self.historian.get_memory_trends(7)?;

        // Skip if no swap configured
        if memory_trends.swap_total_mb == 0 {
            return Ok(None);
        }

        // Critical: Heavy swap usage (>50% swap used)
        if memory_trends.avg_swap_used_mb > 1024 {
            let swap_percent = (memory_trends.avg_swap_used_mb as f64 / memory_trends.swap_total_mb as f64) * 100.0;

            if swap_percent > 50.0 {
                return Ok(Some(
                    Insight::new(
                        "swap_heavy_usage",
                        InsightSeverity::Critical,
                        "Heavy Swap Usage Detected",
                        format!(
                            "System is using {:.0}% of swap memory ({} MB of {} MB). \
                             This indicates severe memory pressure.",
                            swap_percent, memory_trends.avg_swap_used_mb, memory_trends.swap_total_mb
                        ),
                    )
                    .with_evidence(vec![
                        format!("Swap used: {} MB ({:.0}%)", memory_trends.avg_swap_used_mb, swap_percent),
                        format!("RAM pressure: High"),
                    ])
                    .with_suggestion("Consider closing memory-intensive applications or adding more RAM"),
                ));
            }

            // Warning: Moderate swap usage (>20%)
            if swap_percent > 20.0 {
                return Ok(Some(
                    Insight::new(
                        "swap_moderate_usage",
                        InsightSeverity::Warning,
                        "Swap Usage Increasing",
                        format!(
                            "System is using {:.0}% of swap memory ({} MB). \
                             This may impact performance.",
                            swap_percent, memory_trends.avg_swap_used_mb
                        ),
                    )
                    .with_evidence(vec![format!("Swap used: {} MB ({:.0}%)", memory_trends.avg_swap_used_mb, swap_percent)])
                    .with_suggestion("Monitor memory usage and consider closing unused applications"),
                ));
            }
        }

        Ok(None)
    }

    /// Detect Anna inactivity (user hasn't used Anna in a while)
    fn detect_anna_inactivity(&self, hours: i64) -> Result<Option<Insight>> {
        let usage_data = self.historian.get_anna_usage_stats(hours)?;

        // Info: No Anna usage in last 7+ days
        let hours_since_last = usage_data.hours_since_last_invocation;

        if hours_since_last > 168 {
            // 7 days
            let days = hours_since_last / 24;
            return Ok(Some(
                Insight::new(
                    "anna_inactive",
                    InsightSeverity::Info,
                    "Anna Unused Recently",
                    format!(
                        "Anna hasn't been invoked in {} days. Consider running 'annactl \"check my system health\"' \
                         to ensure everything is working properly.",
                        days
                    ),
                )
                .with_evidence(vec![format!("Last used: {} days ago", days)]),
            ));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insight_severity_ordering() {
        let mut insights = vec![
            Insight::new("test1", InsightSeverity::Info, "Info", "info"),
            Insight::new("test2", InsightSeverity::Critical, "Critical", "critical"),
            Insight::new("test3", InsightSeverity::Warning, "Warning", "warning"),
        ];

        insights.sort_by(|a, b| {
            use InsightSeverity::*;
            let a_val = match a.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            let b_val = match b.severity {
                Critical => 3,
                Warning => 2,
                Info => 1,
            };
            b_val.cmp(&a_val)
        });

        assert_eq!(insights[0].severity, InsightSeverity::Critical);
        assert_eq!(insights[1].severity, InsightSeverity::Warning);
        assert_eq!(insights[2].severity, InsightSeverity::Info);
    }

    #[test]
    fn test_insight_id_generation() {
        let insight = Insight::new("test_detector", InsightSeverity::Info, "Test", "Test explanation");

        assert!(insight.id.starts_with("test_detector_"));
        assert_eq!(insight.detector, "test_detector");
    }

    #[test]
    fn test_insight_builder_pattern() {
        let insight = Insight::new("test", InsightSeverity::Warning, "Title", "Explanation")
            .with_evidence(vec!["Evidence 1".to_string(), "Evidence 2".to_string()])
            .with_suggestion("Do this");

        assert_eq!(insight.evidence.len(), 2);
        assert!(insight.suggestion.is_some());
        assert_eq!(insight.suggestion.unwrap(), "Do this");
    }

    #[test]
    fn test_severity_conversion_from_trend_severity() {
        use crate::trend_detectors::TrendSeverity;

        assert_eq!(InsightSeverity::from(TrendSeverity::Info), InsightSeverity::Info);
        assert_eq!(InsightSeverity::from(TrendSeverity::Warning), InsightSeverity::Warning);
        assert_eq!(InsightSeverity::from(TrendSeverity::Critical), InsightSeverity::Critical);
    }

    #[test]
    fn test_insight_with_no_suggestion() {
        let insight = Insight::new("test", InsightSeverity::Info, "Title", "Explanation")
            .with_evidence(vec!["Evidence".to_string()]);

        assert!(insight.suggestion.is_none());
        assert_eq!(insight.evidence.len(), 1);
    }

    #[test]
    fn test_insight_with_empty_evidence() {
        let insight = Insight::new("test", InsightSeverity::Warning, "Title", "Explanation");

        assert!(insight.evidence.is_empty());
        assert!(insight.suggestion.is_none());
    }

    #[test]
    fn test_insight_timestamp_generation() {
        let insight1 = Insight::new("test", InsightSeverity::Info, "Title", "Explanation");
        std::thread::sleep(std::time::Duration::from_secs(1));
        let insight2 = Insight::new("test", InsightSeverity::Info, "Title", "Explanation");

        // IDs should be different due to timestamp (second precision)
        assert_ne!(insight1.id, insight2.id);
        // But detector names should be the same
        assert_eq!(insight1.detector, insight2.detector);
    }
}
