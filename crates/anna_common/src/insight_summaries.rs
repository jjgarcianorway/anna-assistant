//! v6.29.0: Insight Summaries Engine
//!
//! The Insight Summaries Engine aggregates insights, predictive diagnostics, and historical
//! metrics into short, human-friendly summaries. Used for broad queries like:
//! - "Give me a summary of my system"
//! - "What should I know today?"
//! - "How is my computer behaving lately?"
//!
//! ## Design Principles
//!
//! 1. **Deterministic**: Pure rules-based logic, no LLM dependencies
//! 2. **High-Level**: Summaries are conceptual, not low-level logs
//! 3. **Severity-Aware**: Critical issues highlighted first
//! 4. **Detail-Adaptive**: Respects user preferences (Short/Normal/Verbose)
//! 5. **Non-Contradictory**: Never conflicts with source data
//! 6. **Concise**: No verbatim repetition of full insights
//!
//! ## Architecture
//!
//! - Merges 3 data sources: Current insights, Predictive diagnostics, Historical trends
//! - Sorts by severity (Critical → Warning → Info)
//! - Formats according to detail preference
//! - Returns single summary block

use crate::historian::Historian;
use crate::insights_engine::{Insight, InsightSeverity};
use crate::predictive_diagnostics::PredictiveInsight;
use crate::session_context::DetailPreference;
use anyhow::Result;

/// Generate a high-level summary of system insights, predictions, and trends
///
/// This function aggregates multiple data sources into a single concise summary:
/// - Current insights from the Insights Engine
/// - Predictive diagnostics from the Predictive Diagnostics Engine
/// - Historical trends from the Historian
///
/// The output format adapts to user detail preferences:
/// - **Short**: 1-2 lines, critical items only
/// - **Normal**: 4-6 lines, key issues and trends
/// - **Verbose**: 8-12 lines, comprehensive overview
///
/// Returns None if there are no insights, predictions, or significant trends to report.
pub fn generate_insight_summary(
    insights: &[Insight],
    predictive: &[PredictiveInsight],
    historian: &Historian,
    preferences: &DetailPreference,
) -> Result<Option<String>> {
    // If no data at all, return None
    if insights.is_empty() && predictive.is_empty() {
        return Ok(None);
    }

    // Collect and categorize items by severity
    let mut critical_items = Vec::new();
    let mut warning_items = Vec::new();
    let mut info_items = Vec::new();

    // Process insights
    for insight in insights {
        let item = SummaryItem {
            severity: insight.severity,
            category: categorize_insight_title(&insight.title),
            summary: summarize_insight(insight),
        };

        match item.severity {
            InsightSeverity::Critical => critical_items.push(item),
            InsightSeverity::Warning => warning_items.push(item),
            InsightSeverity::Info => info_items.push(item),
        }
    }

    // Process predictions
    for prediction in predictive {
        let item = SummaryItem {
            severity: prediction.severity,
            category: categorize_prediction_title(&prediction.title),
            summary: summarize_prediction(prediction),
        };

        match item.severity {
            InsightSeverity::Critical => critical_items.push(item),
            InsightSeverity::Warning => warning_items.push(item),
            InsightSeverity::Info => info_items.push(item),
        }
    }

    // Add historical trend observations (if verbose)
    if matches!(preferences, DetailPreference::Verbose) {
        if let Some(trend_item) = generate_trend_observation(historian)? {
            info_items.push(trend_item);
        }
    }

    // Format according to detail preference
    let summary = match preferences {
        DetailPreference::Short => format_short_summary(&critical_items, &warning_items),
        DetailPreference::Normal => format_normal_summary(&critical_items, &warning_items, &info_items),
        DetailPreference::Verbose => format_verbose_summary(&critical_items, &warning_items, &info_items),
    };

    if summary.is_empty() {
        Ok(None)
    } else {
        Ok(Some(summary))
    }
}

/// v6.30.0: Generate insight summary with optimization profile
///
/// Respects OptimizationProfile rules:
/// - Filters out suppressed (noisy) insights
/// - Highlights high-value insights (shown even in Short mode)
/// - Uses profile's preferred detail level
pub fn generate_insight_summary_with_optimization(
    insights: &[Insight],
    predictive: &[PredictiveInsight],
    historian: &Historian,
    profile: &crate::optimization_engine::OptimizationProfile,
) -> Result<Option<String>> {
    // Filter insights based on optimization profile
    let filtered_insights: Vec<Insight> = insights
        .iter()
        .filter(|i| !profile.should_suppress(i))
        .cloned()
        .collect();

    // Always include highlighted insights (even in Short mode)
    let highlighted: Vec<&Insight> = filtered_insights
        .iter()
        .filter(|i| profile.should_highlight(i))
        .collect();

    // Use profile's preferred detail level
    let preferences = &profile.preferred_detail;

    // If Short mode but we have highlighted items, upgrade to Normal temporarily
    let effective_preference = if matches!(preferences, DetailPreference::Short) && !highlighted.is_empty() {
        DetailPreference::Normal
    } else {
        preferences.clone()
    };

    generate_insight_summary(&filtered_insights, predictive, historian, &effective_preference)
}

/// Internal representation of a summary item
#[derive(Debug, Clone)]
struct SummaryItem {
    severity: InsightSeverity,
    category: ItemCategory,
    summary: String,
}

/// Categories for grouping related items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemCategory {
    Disk,
    Memory,
    CPU,
    Network,
    Service,
    Boot,
    Thermal,
    IO,
    General,
}

/// Categorize insight by title keywords
fn categorize_insight_title(title: &str) -> ItemCategory {
    let lower = title.to_lowercase();

    if lower.contains("disk") || lower.contains("storage") || lower.contains("space") {
        ItemCategory::Disk
    } else if lower.contains("memory") || lower.contains("ram") || lower.contains("swap") {
        ItemCategory::Memory
    } else if lower.contains("cpu") || lower.contains("processor") {
        ItemCategory::CPU
    } else if lower.contains("network") || lower.contains("latency") || lower.contains("connectivity") {
        ItemCategory::Network
    } else if lower.contains("service") || lower.contains("systemd") || lower.contains("daemon") {
        ItemCategory::Service
    } else if lower.contains("boot") || lower.contains("startup") {
        ItemCategory::Boot
    } else if lower.contains("thermal") || lower.contains("temperature") || lower.contains("cooling") {
        ItemCategory::Thermal
    } else if lower.contains("i/o") || lower.contains("io") {
        ItemCategory::IO
    } else {
        ItemCategory::General
    }
}

/// Categorize prediction by title keywords
fn categorize_prediction_title(title: &str) -> ItemCategory {
    let lower = title.to_lowercase();

    if lower.contains("disk") || lower.contains("storage") || lower.contains("full") {
        ItemCategory::Disk
    } else if lower.contains("thermal") || lower.contains("temperature") || lower.contains("throttling") {
        ItemCategory::Thermal
    } else if lower.contains("cpu") || lower.contains("processor") || lower.contains("pressure") {
        ItemCategory::CPU
    } else if lower.contains("i/o") || lower.contains("wait") {
        ItemCategory::IO
    } else if lower.contains("network") || lower.contains("latency") {
        ItemCategory::Network
    } else {
        ItemCategory::General
    }
}

/// Summarize an insight into a concise one-liner
fn summarize_insight(insight: &Insight) -> String {
    // Extract key info from title and explanation
    let title = &insight.title;

    // Try to extract numerical evidence if present
    let mut numeric_info = None;
    for evidence in &insight.evidence {
        if evidence.contains('%') || evidence.contains("GB") || evidence.contains("ms") {
            numeric_info = Some(evidence.clone());
            break;
        }
    }

    if let Some(info) = numeric_info {
        format!("{} ({})", title, info)
    } else {
        title.clone()
    }
}

/// Summarize a prediction into a concise one-liner
fn summarize_prediction(prediction: &PredictiveInsight) -> String {
    format!("{} ({})", prediction.title, prediction.prediction_window)
}

/// Generate a trend observation from historical data
fn generate_trend_observation(historian: &Historian) -> Result<Option<SummaryItem>> {
    // Get boot trends to see if system is improving/degrading
    let boot_trends = historian.get_boot_trends(7)?;

    let observation = match boot_trends.trend {
        crate::historian::Trend::Up => "Boot time degrading over last 7 days",
        crate::historian::Trend::Down => "Boot time improving over last 7 days",
        crate::historian::Trend::Flat => return Ok(None), // No significant trend
    };

    Ok(Some(SummaryItem {
        severity: InsightSeverity::Info,
        category: ItemCategory::Boot,
        summary: observation.to_string(),
    }))
}

/// Format short summary (1-2 lines, critical + warnings only)
fn format_short_summary(critical: &[SummaryItem], warnings: &[SummaryItem]) -> String {
    if critical.is_empty() && warnings.is_empty() {
        return "Your system is generally healthy. No critical issues detected.".to_string();
    }

    let mut parts = Vec::new();

    if !critical.is_empty() {
        let critical_summary = critical.iter()
            .map(|item| extract_key_phrase(&item.summary))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("Critical: {}", critical_summary));
    }

    if !warnings.is_empty() && critical.len() < 2 {
        let warning_summary = warnings.iter()
            .take(2 - critical.len())
            .map(|item| extract_key_phrase(&item.summary))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("Watch: {}", warning_summary));
    }

    parts.join(". ")
}

/// Format normal summary (4-6 lines, critical + warnings + select info)
fn format_normal_summary(
    critical: &[SummaryItem],
    warnings: &[SummaryItem],
    info: &[SummaryItem],
) -> String {
    if critical.is_empty() && warnings.is_empty() && info.is_empty() {
        return "Your system is operating normally. No issues detected.".to_string();
    }

    let mut lines = Vec::new();

    // Add critical items first
    for item in critical.iter().take(2) {
        lines.push(format!("⚠ Critical: {}", item.summary));
    }

    // Add warning items
    for item in warnings.iter().take(3) {
        lines.push(format!("⚡ Warning: {}", item.summary));
    }

    // Add one info item if space available
    if lines.len() < 5 && !info.is_empty() {
        lines.push(format!("ℹ Info: {}", info[0].summary));
    }

    // If nothing to report but we have info items
    if lines.is_empty() && !info.is_empty() {
        lines.push("System healthy overall.".to_string());
        for item in info.iter().take(2) {
            lines.push(format!("• {}", item.summary));
        }
    }

    lines.join("\n")
}

/// Format verbose summary (8-12 lines, comprehensive overview)
fn format_verbose_summary(
    critical: &[SummaryItem],
    warnings: &[SummaryItem],
    info: &[SummaryItem],
) -> String {
    let mut lines = Vec::new();

    // Header line
    let status = if !critical.is_empty() {
        "⚠ System Health: Critical Issues Detected"
    } else if !warnings.is_empty() {
        "⚡ System Health: Warnings Present"
    } else {
        "✓ System Health: Normal Operation"
    };
    lines.push(status.to_string());
    lines.push(String::new()); // Blank line

    // Critical section
    if !critical.is_empty() {
        lines.push("Critical Issues:".to_string());
        for item in critical {
            lines.push(format!("  • {}", item.summary));
        }
        lines.push(String::new());
    }

    // Warning section
    if !warnings.is_empty() {
        lines.push("Warnings:".to_string());
        for item in warnings.iter().take(5) {
            lines.push(format!("  • {}", item.summary));
        }
        lines.push(String::new());
    }

    // Info section
    if !info.is_empty() {
        lines.push("Observations:".to_string());
        for item in info.iter().take(5) {
            lines.push(format!("  • {}", item.summary));
        }
    }

    // Remove trailing blank line if present
    if lines.last().map(|s| s.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines.join("\n")
}

/// Extract key phrase from a summary (for short format)
fn extract_key_phrase(summary: &str) -> String {
    // Take everything before first parenthesis or comma, or first 40 chars
    let before_paren = summary.split('(').next().unwrap_or(summary);
    let before_comma = before_paren.split(',').next().unwrap_or(before_paren);
    let trimmed = before_comma.trim();

    if trimmed.len() > 40 {
        format!("{}...", &trimmed[..37])
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_generate_summary_empty_data() {
        let insights = vec![];
        let predictive = vec![];
        let historian = create_test_historian();
        let prefs = DetailPreference::Normal;

        let result = generate_insight_summary(&insights, &predictive, &historian, &prefs).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_categorize_disk_insight() {
        assert_eq!(categorize_insight_title("Disk space running low"), ItemCategory::Disk);
        assert_eq!(categorize_insight_title("Storage usage at 85%"), ItemCategory::Disk);
    }

    #[test]
    fn test_categorize_cpu_prediction() {
        assert_eq!(categorize_prediction_title("CPU pressure increasing"), ItemCategory::CPU);
        assert_eq!(categorize_prediction_title("High processor load detected"), ItemCategory::CPU);
    }

    #[test]
    fn test_categorize_thermal_prediction() {
        assert_eq!(categorize_prediction_title("Thermal creep detected"), ItemCategory::Thermal);
        assert_eq!(categorize_prediction_title("Temperature trending upward"), ItemCategory::Thermal);
    }

    #[test]
    fn test_extract_key_phrase() {
        assert_eq!(extract_key_phrase("Disk full (95% capacity)"), "Disk full");
        assert_eq!(extract_key_phrase("Memory pressure, swap active"), "Memory pressure");
        assert_eq!(
            extract_key_phrase("Very long summary that exceeds forty characters and needs truncation"),
            "Very long summary that exceeds forty ..."
        );
    }

    #[test]
    fn test_summarize_insight_with_evidence() {
        let insight = Insight {
            id: "test_1".to_string(),
            timestamp: Utc::now(),
            severity: InsightSeverity::Warning,
            title: "Disk space low".to_string(),
            explanation: "Usage at 85%".to_string(),
            evidence: vec!["Current: 85%".to_string()],
            suggestion: None,
            detector: "disk".to_string(),
        };

        let summary = summarize_insight(&insight);
        assert!(summary.contains("Disk space low"));
        assert!(summary.contains("85%"));
    }

    #[test]
    fn test_format_short_summary_healthy() {
        let critical = vec![];
        let warnings = vec![];

        let summary = format_short_summary(&critical, &warnings);
        assert!(summary.contains("generally healthy"));
    }

    #[test]
    fn test_format_short_summary_with_issues() {
        let critical = vec![SummaryItem {
            severity: InsightSeverity::Critical,
            category: ItemCategory::Disk,
            summary: "Disk full (98% capacity)".to_string(),
        }];
        let warnings = vec![SummaryItem {
            severity: InsightSeverity::Warning,
            category: ItemCategory::CPU,
            summary: "CPU pressure elevated".to_string(),
        }];

        let summary = format_short_summary(&critical, &warnings);
        assert!(summary.contains("Critical"));
        assert!(summary.contains("Disk full"));
    }

    // Helper function to create a test historian
    fn create_test_historian() -> Historian {
        // For testing, we just need a valid historian instance
        // In real usage, it would have actual data
        Historian::new("/tmp/test_historian.db").unwrap()
    }
}
