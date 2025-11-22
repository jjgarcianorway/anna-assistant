// Beta.257: Health Consistency Regression Tests
//
// Purpose: Ensure health/status answers are consistent across all surfaces
// - No contradictions between status and diagnostic answers
// - "Today" wording consistency
// - Icons vs severity consistency

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use annactl::diagnostic_formatter::{compute_overall_health, format_diagnostic_report_with_query, DiagnosticMode, OverallHealth};

/// Test helper: Create a BrainAnalysisData with specified counts
fn mock_analysis(critical_count: usize, warning_count: usize) -> BrainAnalysisData {
    let mut insights = Vec::new();

    // Add critical diagnostics
    for i in 0..critical_count {
        insights.push(DiagnosticInsightData {
            rule_id: format!("critical_rule_{}", i),
            severity: "critical".to_string(),
            summary: format!("Critical issue {}", i),
            details: format!("Critical details {}", i),
            commands: vec![],
            citations: vec![],
            evidence: String::new(),
        });
    }

    // Add warning diagnostics
    for i in 0..warning_count {
        insights.push(DiagnosticInsightData {
            rule_id: format!("warning_rule_{}", i),
            severity: "warning".to_string(),
            summary: format!("Warning issue {}", i),
            details: format!("Warning details {}", i),
            commands: vec![],
            citations: vec![],
            evidence: String::new(),
        });
    }

    BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count,
        warning_count,
        insights,
    }
}

#[test]
fn test_overall_health_healthy() {
    let analysis = mock_analysis(0, 0);
    let health = compute_overall_health(&analysis);

    assert_eq!(health, OverallHealth::Healthy);
}

#[test]
fn test_overall_health_degraded_warning() {
    let analysis = mock_analysis(0, 2);
    let health = compute_overall_health(&analysis);

    assert_eq!(health, OverallHealth::DegradedWarning);
}

#[test]
fn test_overall_health_degraded_critical() {
    let analysis = mock_analysis(1, 0);
    let health = compute_overall_health(&analysis);

    assert_eq!(health, OverallHealth::DegradedCritical);
}

#[test]
fn test_overall_health_critical_overrides_warning() {
    let analysis = mock_analysis(1, 2);
    let health = compute_overall_health(&analysis);

    assert_eq!(health, OverallHealth::DegradedCritical);
}

#[test]
fn test_diagnostic_report_healthy_no_contradictions() {
    let analysis = mock_analysis(0, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Should mention "all clear" or "no...issues"
    let report_lower = report.to_lowercase();
    assert!(report_lower.contains("all clear") || report_lower.contains("no") && report_lower.contains("issues"),
            "Report should mention 'all clear' or 'no issues'");
    assert!(!report_lower.contains("warning(s) detected"), "Healthy system should not mention warnings detected");
    assert!(!report_lower.contains("issue(s) detected"), "Healthy system should not mention issues detected");
}

#[test]
fn test_diagnostic_report_degraded_warning_no_contradictions() {
    let analysis = mock_analysis(0, 2);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Should contain "warning(s) detected" and not critical
    let report_lower = report.to_lowercase();
    assert!(report_lower.contains("warning"), "Warning-level system should mention 'warning'");
    assert!(report_lower.contains("no critical issues"), "Warning-only system should state no critical issues");
    assert!(!report_lower.contains("all clear"), "Warning-level system should not say 'all clear'");
}

#[test]
fn test_diagnostic_report_degraded_critical_no_contradictions() {
    let analysis = mock_analysis(1, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Should contain "issue(s) detected" and "critical"
    let report_lower = report.to_lowercase();
    assert!(report_lower.contains("issue(s) detected"), "System should mention 'issue(s) detected'");
    assert!(report_lower.contains("critical"), "Critical-level system should mention 'critical'");
    assert!(!report_lower.contains("all clear"), "Critical system should not say 'all clear'");
}

#[test]
fn test_today_wording_temporal_query() {
    let analysis = mock_analysis(0, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "how is my system today");

    // Should use temporal wording
    assert!(report.contains("System health today:"),
            "Query with 'today' should use temporal wording");
}

#[test]
fn test_today_wording_recently_query() {
    let analysis = mock_analysis(0, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "any errors recently");

    // Should use temporal wording
    assert!(report.contains("System health today:"),
            "Query with 'recently' should use temporal wording");
}

#[test]
fn test_today_wording_generic_query() {
    let analysis = mock_analysis(0, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Should use generic wording
    assert!(report.contains("System health:") && !report.contains("System health today:"),
            "Generic query should use non-temporal wording");
}

#[test]
fn test_icon_severity_consistency_healthy() {
    let analysis = mock_analysis(0, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Healthy systems have no insights, so no severity icons
    // But should contain canonical structure markers
    assert!(report.contains("[SUMMARY]"), "Report should have [SUMMARY] section");
    assert!(report.contains("[COMMANDS]"), "Report should have [COMMANDS] section");
    // Should NOT contain severity icons (no issues to show)
    assert!(!report.contains("✗"), "Healthy report should not contain critical icon");
    assert!(!report.contains("⚠"), "Healthy report should not contain warning icon");
}

#[test]
fn test_icon_severity_consistency_degraded() {
    let analysis = mock_analysis(1, 0);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Degraded with critical should use ✗ icon
    assert!(report.contains("✗"), "Degraded report with critical should contain ✗ icon");
    assert!(report.contains("[DETAILS]"), "Report with issues should have [DETAILS] section");
}

#[test]
fn test_icon_severity_consistency_warning() {
    let analysis = mock_analysis(0, 1);
    let report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // Degraded with warning should use ⚠ icon
    assert!(report.contains("⚠"), "Degraded report with warning should contain ⚠ icon");
    assert!(report.contains("[DETAILS]"), "Report with issues should have [DETAILS] section");
}
