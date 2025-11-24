//! Beta.250: Canonical Diagnostic Formatter
//!
//! Single source of truth for formatting diagnostic engine results.
//! Used by:
//! - One-shot diagnostic queries (`annactl "check my system health"`)
//! - Status command (`annactl status`)
//! - TUI diagnostic panel
//!
//! Two modes:
//! - Full mode: Complete diagnostic report with all insights
//! - Summary mode: Top N issues for status/TUI display

use anna_common::ipc::BrainAnalysisData;
use std::fmt::Write;

/// Beta.257: Overall system health level
///
/// Single source of truth for health status, computed from diagnostic engine output.
/// Used across all surfaces (status, diagnostics, TUI) to ensure consistent messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverallHealth {
    /// System is healthy - no critical issues, no warnings
    #[default]
    Healthy,
    /// System is degraded with warnings - no critical issues, but warnings present
    DegradedWarning,
    /// System is degraded with critical issues - one or more critical issues present
    DegradedCritical,
}

/// Beta.257: Compute overall health level from diagnostic data
///
/// Deterministic health computation based on diagnostic engine results:
/// - Critical issues present → DegradedCritical
/// - Warnings only → DegradedWarning
/// - No issues → Healthy
///
/// This is the single source of truth for health status across all surfaces.
pub fn compute_overall_health(analysis: &BrainAnalysisData) -> OverallHealth {
    if analysis.critical_count > 0 {
        OverallHealth::DegradedCritical
    } else if analysis.warning_count > 0 {
        OverallHealth::DegradedWarning
    } else {
        OverallHealth::Healthy
    }
}

/// Beta.259: Lightweight daily snapshot for TUI state
///
/// Simplified version of DailySnapshot with only the essential fields
/// that TUI needs to display health status.
#[derive(Debug, Clone, Default)]
pub struct DailySnapshotLite {
    /// Overall system health level
    pub overall_health_opt: Option<OverallHealth>,
    /// Critical issues count
    pub critical_count: usize,
    /// Warning issues count
    pub warning_count: usize,
    /// Kernel changed since last session
    pub kernel_changed: bool,
    /// Package count changed
    pub packages_changed: bool,
    /// Number of boots since last session
    pub boots_since_last: u32,
}

impl DailySnapshotLite {
    /// Create from full DailySnapshot
    pub fn from_snapshot(snapshot: &DailySnapshot) -> Self {
        Self {
            overall_health_opt: Some(snapshot.overall_health),
            critical_count: snapshot.critical_count,
            warning_count: snapshot.warning_count,
            kernel_changed: snapshot.kernel_changed,
            packages_changed: snapshot.package_delta != 0,
            boots_since_last: snapshot.boots_since_last,
        }
    }
}

/// Beta.258: Daily snapshot data combining health and session delta
///
/// Compact struct for "How is my system today?" queries.
/// Merges diagnostic engine output with session metadata deltas.
#[derive(Debug, Clone)]
pub struct DailySnapshot {
    /// Overall system health level
    pub overall_health: OverallHealth,
    /// Critical issues count
    pub critical_count: usize,
    /// Warning issues count
    pub warning_count: usize,
    /// Top 3 issue summaries for brief display
    pub top_issue_summaries: Vec<String>,
    /// Kernel changed since last session
    pub kernel_changed: bool,
    /// Old kernel version (if changed)
    pub old_kernel: Option<String>,
    /// New kernel version (if changed)
    pub new_kernel: Option<String>,
    /// Package count changed since last session
    pub package_delta: i32,
    /// Number of boots since last session (0 or 1+)
    pub boots_since_last: u32,
}

/// Beta.258: Session delta information extracted from telemetry comparison
#[derive(Debug, Clone, Default)]
pub struct SessionDelta {
    /// Kernel changed since last session
    pub kernel_changed: bool,
    /// Old kernel version
    pub old_kernel: Option<String>,
    /// New kernel version
    pub new_kernel: Option<String>,
    /// Package count delta (positive = upgrades, negative = removals)
    pub package_delta: i32,
    /// Boots since last session (derived from kernel change)
    pub boots_since_last: u32,
}

/// Beta.258: Compute daily snapshot from diagnostic and session data
///
/// Combines:
/// - Diagnostic engine output (health, issues)
/// - Session metadata delta (kernel, packages, boots)
///
/// Returns a compact snapshot suitable for "today" queries.
pub fn compute_daily_snapshot(
    analysis: &BrainAnalysisData,
    session_delta: SessionDelta,
) -> DailySnapshot {
    let overall_health = compute_overall_health(analysis);

    // Extract top 3 issue summaries
    let top_issue_summaries: Vec<String> = analysis
        .insights
        .iter()
        .take(3)
        .map(|insight| insight.summary.clone())
        .collect();

    DailySnapshot {
        overall_health,
        critical_count: analysis.critical_count,
        warning_count: analysis.warning_count,
        top_issue_summaries,
        kernel_changed: session_delta.kernel_changed,
        old_kernel: session_delta.old_kernel,
        new_kernel: session_delta.new_kernel,
        package_delta: session_delta.package_delta,
        boots_since_last: session_delta.boots_since_last,
    }
}

/// Formatting mode for diagnostic reports
#[derive(Debug, Clone, Copy)]
pub enum DiagnosticMode {
    /// Full diagnostic report - shows all insights (up to 5)
    Full,
    /// Summary mode - shows top 3 insights for status/TUI
    Summary,
}

/// Beta.250: Format diagnostic engine results with consistent structure
///
/// Always uses [SUMMARY] / [DETAILS] / [COMMANDS] structure from ANSWER_FORMAT.md
///
/// Key principles:
/// - [SUMMARY] explicitly states critical/warning counts or "all clear"
/// - [DETAILS] lists issues ordered by severity (critical → warning → info)
/// - [COMMANDS] shows prioritized, actionable commands
/// - Severity markers: ✗ (critical), ⚠ (warning), ℹ (info)
pub fn format_diagnostic_report(analysis: &BrainAnalysisData, mode: DiagnosticMode) -> String {
    let mut report = String::new();

    // Determine max issues to show based on mode
    let max_issues = match mode {
        DiagnosticMode::Full => 5,
        DiagnosticMode::Summary => 3,
    };

    // [SUMMARY] - Always clearly state health status
    writeln!(&mut report, "[SUMMARY]").unwrap();

    if analysis.critical_count == 0 && analysis.warning_count == 0 {
        writeln!(&mut report, "System health: **all clear, no critical issues detected.**").unwrap();
        if matches!(mode, DiagnosticMode::Full) {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "All diagnostic checks passed. System is operating normally.").unwrap();
        }
    } else if analysis.critical_count > 0 {
        writeln!(
            &mut report,
            "System health: **{} issue(s) detected: {} critical, {} warning(s).**",
            analysis.critical_count + analysis.warning_count,
            analysis.critical_count,
            analysis.warning_count
        ).unwrap();
        if matches!(mode, DiagnosticMode::Full) {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "Immediate attention required for critical issues.").unwrap();
        }
    } else {
        writeln!(
            &mut report,
            "System health: **{} warning(s) detected, no critical issues.**",
            analysis.warning_count
        ).unwrap();
        if matches!(mode, DiagnosticMode::Full) {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "System is stable but warnings should be investigated.").unwrap();
        }
    }

    // Beta.271: Add proactive issue count if present
    if !analysis.proactive_issues.is_empty() {
        writeln!(
            &mut report,
            "ℹ Proactive engine detected {} correlated issue pattern(s).",
            analysis.proactive_issues.len()
        ).unwrap();
    }
    writeln!(&mut report).unwrap();

    // Beta.273: [PROACTIVE SUMMARY] - First-class proactive status
    if !analysis.proactive_issues.is_empty() {
        writeln!(&mut report, "[PROACTIVE SUMMARY]").unwrap();
        writeln!(
            &mut report,
            "- {} correlated issue(s) detected",
            analysis.proactive_issues.len()
        ).unwrap();
        writeln!(
            &mut report,
            "- Health score: {}/100",
            analysis.proactive_health_score
        ).unwrap();

        // Get top issue by severity
        let mut sorted_issues = analysis.proactive_issues.clone();
        sorted_issues.sort_by(|a, b| {
            let a_priority = severity_priority_proactive(&a.severity);
            let b_priority = severity_priority_proactive(&b.severity);
            b_priority.cmp(&a_priority) // Descending
        });

        if let Some(top_issue) = sorted_issues.first() {
            writeln!(
                &mut report,
                "- Top root cause: {}",
                top_issue.root_cause
            ).unwrap();
        }
        writeln!(&mut report).unwrap();
    }

    // [DETAILS] - Show insights, limited by mode
    if !analysis.insights.is_empty() {
        writeln!(&mut report, "[DETAILS]").unwrap();
        writeln!(&mut report).unwrap();

        let insights_to_show = analysis.insights.iter().take(max_issues);

        for (idx, insight) in insights_to_show.enumerate() {
            let severity_marker = match insight.severity.to_lowercase().as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };

            writeln!(&mut report, "{}. {} **{}**", idx + 1, severity_marker, insight.summary).unwrap();

            if matches!(mode, DiagnosticMode::Full) {
                writeln!(&mut report, "   {}", insight.details).unwrap();
                writeln!(&mut report).unwrap();

                // Add diagnostic commands if available
                if !insight.commands.is_empty() {
                    for cmd in &insight.commands {
                        writeln!(&mut report, "   $ {}", cmd).unwrap();
                    }
                    writeln!(&mut report).unwrap();
                }
            } else {
                // Summary mode: Just show one-line description, no commands per insight
                writeln!(&mut report).unwrap();
            }
        }

        // In summary mode, show count of remaining issues
        if matches!(mode, DiagnosticMode::Summary) && analysis.insights.len() > max_issues {
            writeln!(
                &mut report,
                "... and {} more (run 'annactl \"check my system health\"' for full analysis)",
                analysis.insights.len() - max_issues
            ).unwrap();
            writeln!(&mut report).unwrap();
        }
    }

    // Beta.272: [PROACTIVE] - Correlated issues from proactive engine
    if !analysis.proactive_issues.is_empty() {
        writeln!(&mut report, "[PROACTIVE]").unwrap();
        writeln!(&mut report, "Top correlated issues:").unwrap();

        // Sort by severity (critical first) and cap at 10
        let mut sorted_issues = analysis.proactive_issues.clone();
        sorted_issues.sort_by(|a, b| {
            let a_priority = severity_priority_proactive(&a.severity);
            let b_priority = severity_priority_proactive(&b.severity);
            b_priority.cmp(&a_priority) // Descending (higher priority first)
        });
        sorted_issues.truncate(10);

        for (idx, issue) in sorted_issues.iter().enumerate() {
            let marker = match issue.severity.to_lowercase().as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };
            writeln!(&mut report, "{}. {} {}", idx + 1, marker, issue.summary).unwrap();
        }
        writeln!(&mut report).unwrap();
    }

    // [COMMANDS] - Prioritized action list
    writeln!(&mut report, "[COMMANDS]").unwrap();
    if analysis.critical_count > 0 || analysis.warning_count > 0 {
        writeln!(&mut report).unwrap();
        writeln!(&mut report, "$ annactl status").unwrap();
        writeln!(&mut report, "$ journalctl -xe").unwrap();
        writeln!(&mut report, "$ systemctl --failed").unwrap();

        // Beta.267: Add network hint if network issues present
        let has_network_issues = analysis.insights.iter().any(|i|
            i.rule_id.starts_with("network_") ||
            i.rule_id.contains("packet_loss") ||
            i.rule_id.contains("latency")
        );
        if has_network_issues {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "# Network issues detected - run focused diagnostic:").unwrap();
            writeln!(&mut report, "$ annactl \"check my network\"").unwrap();
        }
    } else {
        writeln!(&mut report).unwrap();
        writeln!(&mut report, "No actions required - system is healthy.").unwrap();
        writeln!(&mut report).unwrap();
        writeln!(&mut report, "$ annactl status        # View current status").unwrap();
    }

    report
}

/// Beta.257: Format diagnostic report with query-aware wording
///
/// Adds "today" or "recently" to health summary when query contains temporal terms.
/// This provides more natural responses to queries like "How is my system today?"
pub fn format_diagnostic_report_with_query(
    analysis: &BrainAnalysisData,
    mode: DiagnosticMode,
    query: &str,
) -> String {
    let normalized_query = query.to_lowercase();
    let use_temporal_wording = normalized_query.contains("today") || normalized_query.contains("recently");

    let mut report = String::new();

    // Determine max issues to show based on mode
    let max_issues = match mode {
        DiagnosticMode::Full => 5,
        DiagnosticMode::Summary => 3,
    };

    // [SUMMARY] - Health status with optional temporal qualifier
    writeln!(&mut report, "[SUMMARY]").unwrap();

    let health_prefix = if use_temporal_wording {
        "System health today:"
    } else {
        "System health:"
    };

    if analysis.critical_count == 0 && analysis.warning_count == 0 {
        writeln!(&mut report, "{} **all clear, no critical issues detected.**", health_prefix).unwrap();
        if matches!(mode, DiagnosticMode::Full) {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "All diagnostic checks passed. System is operating normally.").unwrap();
        }
    } else if analysis.critical_count > 0 {
        writeln!(
            &mut report,
            "{} **{} issue(s) detected: {} critical, {} warning(s).**",
            health_prefix,
            analysis.critical_count + analysis.warning_count,
            analysis.critical_count,
            analysis.warning_count
        ).unwrap();
        if matches!(mode, DiagnosticMode::Full) {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "Immediate attention required for critical issues.").unwrap();
        }
    } else {
        writeln!(
            &mut report,
            "{} **{} warning(s) detected, no critical issues.**",
            health_prefix,
            analysis.warning_count
        ).unwrap();
        if matches!(mode, DiagnosticMode::Full) {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "System is stable but warnings should be investigated.").unwrap();
        }
    }

    // Beta.271: Add proactive issue count if present
    if !analysis.proactive_issues.is_empty() {
        writeln!(
            &mut report,
            "ℹ Proactive engine detected {} correlated issue pattern(s).",
            analysis.proactive_issues.len()
        ).unwrap();
    }
    writeln!(&mut report).unwrap();

    // Beta.273: [PROACTIVE SUMMARY] - First-class proactive status
    if !analysis.proactive_issues.is_empty() {
        writeln!(&mut report, "[PROACTIVE SUMMARY]").unwrap();
        writeln!(
            &mut report,
            "- {} correlated issue(s) detected",
            analysis.proactive_issues.len()
        ).unwrap();
        writeln!(
            &mut report,
            "- Health score: {}/100",
            analysis.proactive_health_score
        ).unwrap();

        // Get top issue by severity
        let mut sorted_issues = analysis.proactive_issues.clone();
        sorted_issues.sort_by(|a, b| {
            let a_priority = severity_priority_proactive(&a.severity);
            let b_priority = severity_priority_proactive(&b.severity);
            b_priority.cmp(&a_priority) // Descending
        });

        if let Some(top_issue) = sorted_issues.first() {
            writeln!(
                &mut report,
                "- Top root cause: {}",
                top_issue.root_cause
            ).unwrap();
        }
        writeln!(&mut report).unwrap();
    }

    // [DETAILS] - Show insights, limited by mode
    if !analysis.insights.is_empty() {
        writeln!(&mut report, "[DETAILS]").unwrap();
        writeln!(&mut report).unwrap();

        let insights_to_show = analysis.insights.iter().take(max_issues);

        for (idx, insight) in insights_to_show.enumerate() {
            let severity_marker = match insight.severity.to_lowercase().as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };

            writeln!(&mut report, "{}. {} **{}**", idx + 1, severity_marker, insight.summary).unwrap();

            if matches!(mode, DiagnosticMode::Full) {
                writeln!(&mut report, "   {}", insight.details).unwrap();
                writeln!(&mut report).unwrap();

                if !insight.commands.is_empty() {
                    for cmd in &insight.commands {
                        writeln!(&mut report, "   $ {}", cmd).unwrap();
                    }
                    writeln!(&mut report).unwrap();
                }
            }
        }

        if matches!(mode, DiagnosticMode::Summary) && analysis.insights.len() > max_issues {
            writeln!(
                &mut report,
                "... and {} more (run 'annactl \"check my system health\"' for full analysis)",
                analysis.insights.len() - max_issues
            ).unwrap();
            writeln!(&mut report).unwrap();
        }
    }

    // Beta.272: [PROACTIVE] - Correlated issues from proactive engine
    if !analysis.proactive_issues.is_empty() {
        writeln!(&mut report, "[PROACTIVE]").unwrap();
        writeln!(&mut report, "Top correlated issues:").unwrap();

        // Sort by severity (critical first) and cap at 10
        let mut sorted_issues = analysis.proactive_issues.clone();
        sorted_issues.sort_by(|a, b| {
            let a_priority = severity_priority_proactive(&a.severity);
            let b_priority = severity_priority_proactive(&b.severity);
            b_priority.cmp(&a_priority) // Descending (higher priority first)
        });
        sorted_issues.truncate(10);

        for (idx, issue) in sorted_issues.iter().enumerate() {
            let marker = match issue.severity.to_lowercase().as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };
            writeln!(&mut report, "{}. {} {}", idx + 1, marker, issue.summary).unwrap();
        }
        writeln!(&mut report).unwrap();
    }

    // [COMMANDS] - Prioritized action list
    writeln!(&mut report, "[COMMANDS]").unwrap();
    if analysis.critical_count > 0 || analysis.warning_count > 0 {
        writeln!(&mut report).unwrap();
        writeln!(&mut report, "$ annactl status").unwrap();
        writeln!(&mut report, "$ journalctl -xe").unwrap();
        writeln!(&mut report, "$ systemctl --failed").unwrap();

        // Beta.267: Add network hint if network issues present
        let has_network_issues = analysis.insights.iter().any(|i|
            i.rule_id.starts_with("network_") ||
            i.rule_id.contains("packet_loss") ||
            i.rule_id.contains("latency")
        );
        if has_network_issues {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "# Network issues detected - run focused diagnostic:").unwrap();
            writeln!(&mut report, "$ annactl \"check my network\"").unwrap();
        }
    } else {
        writeln!(&mut report).unwrap();
        writeln!(&mut report, "No actions required - system is healthy.").unwrap();
        writeln!(&mut report).unwrap();
        writeln!(&mut report, "$ annactl status        # View current status").unwrap();
    }

    report
}

/// Beta.250: Format diagnostic summary for inline display (e.g., status command)
///
/// Compact one-line or multi-line format suitable for embedding in status output.
/// Does NOT use [SUMMARY]/[DETAILS]/[COMMANDS] structure - just the core info.
pub fn format_diagnostic_summary_inline(analysis: &BrainAnalysisData) -> String {
    let mut summary = String::new();

    if analysis.critical_count > 0 || analysis.warning_count > 0 {
        writeln!(
            &mut summary,
            "  {} critical, {} warning",
            analysis.critical_count,
            analysis.warning_count
        ).unwrap();
        writeln!(&mut summary).unwrap();

        // Show top 3 insights with concrete details (6.8.1 hotfix)
        for (idx, insight) in analysis.insights.iter().take(3).enumerate() {
            let severity_marker = match insight.severity.to_lowercase().as_str() {
                "critical" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };
            writeln!(
                &mut summary,
                "  {} {}. {}",
                severity_marker,
                idx + 1,
                insight.summary
            ).unwrap();

            // 6.8.1: Show concrete details/evidence, not just summary
            if !insight.evidence.is_empty() {
                // Evidence contains concrete data (filesystem %, log messages, etc)
                writeln!(&mut summary, "     {}", insight.evidence).unwrap();
            } else if !insight.details.is_empty() && insight.details != insight.summary {
                // Fall back to details if no evidence
                writeln!(&mut summary, "     {}", insight.details).unwrap();
            }
        }

        if analysis.insights.len() > 3 {
            writeln!(
                &mut summary,
                "    ... and {} more (run 'annactl \"check my system health\"' for complete analysis)",
                analysis.insights.len() - 3
            ).unwrap();
        }
    } else {
        writeln!(&mut summary, "  All systems nominal").unwrap();
        writeln!(&mut summary, "    No critical issues detected").unwrap();
    }

    summary
}

/// Beta.258: Format daily snapshot for "today" queries
///
/// Produces a short (5-10 line) sysadmin briefing that combines:
/// - Health status with temporal wording if requested
/// - Session deltas (kernel, packages, boots)
/// - Issue counts in compact format
///
/// Does NOT include [COMMANDS] section - this is overview only.
pub fn format_daily_snapshot(snapshot: &DailySnapshot, temporal: bool) -> String {
    let mut report = String::new();

    // Health summary line with optional temporal wording
    let health_prefix = if temporal {
        "System health today:"
    } else {
        "System health:"
    };

    match snapshot.overall_health {
        OverallHealth::Healthy => {
            writeln!(&mut report, "{} **all clear, no critical issues detected.**", health_prefix).unwrap();
        }
        OverallHealth::DegradedWarning => {
            writeln!(&mut report, "{} **degraded – warning issues detected.**", health_prefix).unwrap();
        }
        OverallHealth::DegradedCritical => {
            writeln!(&mut report, "{} **degraded – critical issues require attention.**", health_prefix).unwrap();
        }
    }
    writeln!(&mut report).unwrap();

    // Session delta information
    writeln!(&mut report, "[SESSION DELTA]").unwrap();

    // Kernel status
    if snapshot.kernel_changed {
        if let (Some(old), Some(new)) = (&snapshot.old_kernel, &snapshot.new_kernel) {
            writeln!(&mut report, "- Kernel: updated since last session ({} → {})", old, new).unwrap();
        } else {
            writeln!(&mut report, "- Kernel: updated since last session").unwrap();
        }
    } else {
        writeln!(&mut report, "- Kernel: unchanged since last session").unwrap();
    }

    // Package status
    if snapshot.package_delta > 0 {
        writeln!(&mut report, "- Packages: {} package(s) upgraded", snapshot.package_delta).unwrap();
    } else if snapshot.package_delta < 0 {
        writeln!(&mut report, "- Packages: {} package(s) removed", snapshot.package_delta.abs()).unwrap();
    } else {
        writeln!(&mut report, "- Packages: no changes since last session").unwrap();
    }

    // Boot status
    if snapshot.boots_since_last == 0 {
        writeln!(&mut report, "- Boots: no reboots since last session").unwrap();
    } else if snapshot.boots_since_last == 1 {
        writeln!(&mut report, "- Boots: 1 reboot since last session").unwrap();
    } else {
        writeln!(&mut report, "- Boots: {}+ reboots since last session", snapshot.boots_since_last).unwrap();
    }

    // Issue summary
    if snapshot.critical_count > 0 || snapshot.warning_count > 0 {
        writeln!(
            &mut report,
            "- Issues: {} critical, {} warning(s)",
            snapshot.critical_count,
            snapshot.warning_count
        ).unwrap();

        // Show top issues if present
        if !snapshot.top_issue_summaries.is_empty() {
            writeln!(&mut report).unwrap();
            writeln!(&mut report, "[TOP ISSUES]").unwrap();
            for (idx, summary) in snapshot.top_issue_summaries.iter().enumerate() {
                let marker = if idx < snapshot.critical_count {
                    "✗"
                } else {
                    "⚠"
                };
                writeln!(&mut report, "  {} {}", marker, summary).unwrap();
            }
        }
    } else {
        writeln!(&mut report, "- Issues: 0 critical, 0 warnings").unwrap();
    }

    report
}

/// Beta.259: Format single-line "Today:" health summary from DailySnapshot
///
/// Used by annactl status for the top health line.
/// Returns a string like "System health **all clear, no critical issues detected.**"
pub fn format_today_health_line(snapshot: &DailySnapshot) -> String {
    match snapshot.overall_health {
        OverallHealth::Healthy => "System health **all clear, no critical issues detected.**".to_string(),
        OverallHealth::DegradedWarning => "System health **degraded – warning issues detected.**".to_string(),
        OverallHealth::DegradedCritical => "System health **degraded – critical issues require attention.**".to_string(),
    }
}

/// Beta.259: Format single-line "Today:" health summary from OverallHealth
///
/// Simplified version that takes just the health level.
pub fn format_today_health_line_from_health(health: OverallHealth) -> String {
    match health {
        OverallHealth::Healthy => "System health **all clear, no critical issues detected.**".to_string(),
        OverallHealth::DegradedWarning => "System health **degraded – warning issues detected.**".to_string(),
        OverallHealth::DegradedCritical => "System health **degraded – critical issues require attention.**".to_string(),
    }
}

/// Beta.272: Get severity priority for proactive issues sorting
///
/// Higher number = higher priority (critical > warning > info > trend)
pub fn severity_priority_proactive(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "critical" => 4,
        "warning" => 3,
        "info" => 2,
        "trend" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};

    #[test]
    fn test_format_all_clear_full() {
        let analysis = BrainAnalysisData {
            timestamp: "2025-11-22T12:00:00Z".to_string(),
            insights: vec![],
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
            proactive_issues: vec![],
            proactive_health_score: 100,
        };

        let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

        assert!(report.contains("[SUMMARY]"));
        assert!(report.contains("all clear"));
        assert!(report.contains("no critical issues detected"));
        // When there are no insights, [DETAILS] section is skipped
        assert!(report.contains("[COMMANDS]"));
        assert!(report.contains("No actions required"));
    }

    #[test]
    fn test_format_critical_issues_full() {
        let analysis = BrainAnalysisData {
            timestamp: "2025-11-22T12:00:00Z".to_string(),
            formatted_output: String::new(),
            critical_count: 2,
            warning_count: 1,
            proactive_issues: vec![],
            proactive_health_score: 100,
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "failed_services".to_string(),
                    severity: "critical".to_string(),
                    summary: "Failed service detected".to_string(),
                    details: "Service foo.service is in failed state".to_string(),
                    commands: vec!["systemctl status foo.service".to_string()],
                    citations: vec![],
                    evidence: String::new(),
                },
                DiagnosticInsightData {
                    rule_id: "disk_space".to_string(),
                    severity: "critical".to_string(),
                    summary: "Disk space low".to_string(),
                    details: "Root filesystem at 95% capacity".to_string(),
                    commands: vec!["df -h".to_string()],
                    citations: vec![],
                    evidence: String::new(),
                },
                DiagnosticInsightData {
                    rule_id: "orphans".to_string(),
                    severity: "warning".to_string(),
                    summary: "Orphaned packages found".to_string(),
                    details: "5 orphaned packages detected".to_string(),
                    commands: vec!["pacman -Qtdq".to_string()],
                    citations: vec![],
                    evidence: String::new(),
                },
            ],
        };

        let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

        assert!(report.contains("[SUMMARY]"));
        assert!(report.contains("3 issue(s) detected: 2 critical, 1 warning"));
        assert!(report.contains("Immediate attention required"));
        assert!(report.contains("[DETAILS]"));
        assert!(report.contains("✗")); // Critical marker
        assert!(report.contains("⚠")); // Warning marker
        assert!(report.contains("Failed service detected"));
        assert!(report.contains("Disk space low"));
        assert!(report.contains("Orphaned packages found"));
        assert!(report.contains("[COMMANDS]"));
        assert!(report.contains("systemctl --failed"));
    }

    #[test]
    fn test_format_summary_mode_limits_issues() {
        let mut insights = vec![];
        for i in 1..=5 {
            insights.push(DiagnosticInsightData {
                rule_id: format!("rule_{}", i),
                severity: "critical".to_string(),
                summary: format!("Issue {}", i),
                details: format!("Details {}", i),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            });
        }

        let analysis = BrainAnalysisData {
            timestamp: "2025-11-22T12:00:00Z".to_string(),
            formatted_output: String::new(),
            critical_count: 5,
            warning_count: 0,
            proactive_issues: vec![],
            proactive_health_score: 100,
            insights,
        };

        let report = format_diagnostic_report(&analysis, DiagnosticMode::Summary);

        // Summary mode should only show top 3 issues
        assert!(report.contains("Issue 1"));
        assert!(report.contains("Issue 2"));
        assert!(report.contains("Issue 3"));
        assert!(!report.contains("Issue 4"));
        assert!(!report.contains("Issue 5"));
        assert!(report.contains("... and 2 more"));
    }

    #[test]
    fn test_inline_summary_formatting() {
        let analysis = BrainAnalysisData {
            timestamp: "2025-11-22T12:00:00Z".to_string(),
            formatted_output: String::new(),
            critical_count: 1,
            warning_count: 1,
            proactive_issues: vec![],
            proactive_health_score: 100,
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "rule_1".to_string(),
                    severity: "critical".to_string(),
                    summary: "Critical issue".to_string(),
                    details: "Details".to_string(),
                    commands: vec![],
                    citations: vec![],
                    evidence: String::new(),
                },
                DiagnosticInsightData {
                    rule_id: "rule_2".to_string(),
                    severity: "warning".to_string(),
                    summary: "Warning issue".to_string(),
                    details: "Details".to_string(),
                    commands: vec![],
                    citations: vec![],
                    evidence: String::new(),
                },
            ],
        };

        let summary = format_diagnostic_summary_inline(&analysis);

        assert!(summary.contains("1 critical, 1 warning"));
        assert!(summary.contains("Critical issue"));
        assert!(summary.contains("Warning issue"));
        // Should NOT contain [SUMMARY] tags
        assert!(!summary.contains("[SUMMARY]"));
    }
}
