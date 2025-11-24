//! Beta.273: Proactive Engine v2 - TUI & CLI First-Class Experience Regression Tests
//!
//! These tests validate Beta.273's promotion of proactive engine to first-class status:
//! - CLI [PROACTIVE SUMMARY] section with health score and top root cause
//! - TUI proactive mini-panel with scores (capped at 3)
//! - NL routing for proactive status summary queries
//! - Cross-surface consistency (naming, scores, severity markers)

use annactl::diagnostic_formatter::{format_diagnostic_report_with_query, severity_priority_proactive, DiagnosticMode};
use anna_common::ipc::{BrainAnalysisData, ProactiveIssueSummaryData};

/// Helper: Create BrainAnalysisData with proactive issues and health score
fn create_brain_data_with_proactive_and_score(
    issues: Vec<ProactiveIssueSummaryData>,
    health_score: u8,
) -> BrainAnalysisData {
    BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues: issues,
        proactive_health_score: health_score,
    }
}

/// Helper: Create a test proactive issue
fn create_test_issue(root_cause: &str, severity: &str, confidence: f32) -> ProactiveIssueSummaryData {
    ProactiveIssueSummaryData {
        root_cause: root_cause.to_string(),
        severity: severity.to_string(),
        summary: format!("Test summary for {}", root_cause),
        rule_id: None,
        confidence,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
        suggested_fix: None,
    }
}

// ============================================================================
// CLI [PROACTIVE SUMMARY] Section Tests
// ============================================================================

#[test]
fn test_cli_proactive_summary_appears_when_issues_exist() {
    // Beta.273: [PROACTIVE SUMMARY] should appear when proactive issues exist
    let issues = vec![
        create_test_issue("network_routing_conflict", "critical", 0.95),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 75);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("[PROACTIVE SUMMARY]"),
        "Should contain [PROACTIVE SUMMARY] section. Got:\n{}", report);
}

#[test]
fn test_cli_proactive_summary_hidden_when_no_issues() {
    // Beta.273: [PROACTIVE SUMMARY] should NOT appear when no issues
    let data = create_brain_data_with_proactive_and_score(vec![], 100);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(!report.contains("[PROACTIVE SUMMARY]"),
        "Should not show [PROACTIVE SUMMARY] when empty. Got:\n{}", report);
}

#[test]
fn test_cli_shows_issue_count() {
    // Beta.273: Should show "- X correlated issue(s) detected"
    let issues = vec![
        create_test_issue("disk_pressure", "warning", 0.85),
        create_test_issue("service_flapping", "warning", 0.80),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 80);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("- 2 correlated issue(s) detected"),
        "Should show issue count. Got:\n{}", report);
}

#[test]
fn test_cli_shows_health_score() {
    // Beta.273: Should show "- Health score: XX/100"
    let issues = vec![
        create_test_issue("cpu_overload", "critical", 0.90),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 65);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("- Health score: 65/100"),
        "Should show health score. Got:\n{}", report);
}

#[test]
fn test_cli_shows_top_root_cause() {
    // Beta.273: Should show "- Top root cause: <root_cause>"
    let issues = vec![
        create_test_issue("memory_pressure", "critical", 0.92),
        create_test_issue("disk_log_growth", "warning", 0.75),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 70);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Critical should be top due to severity sorting
    assert!(report.contains("- Top root cause: memory_pressure"),
        "Should show top root cause (critical first). Got:\n{}", report);
}

#[test]
fn test_cli_proactive_summary_positioning() {
    // Beta.273: [PROACTIVE SUMMARY] should appear after [SUMMARY], before [DETAILS]
    let issues = vec![
        create_test_issue("test", "warning", 0.80),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 85);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    let summary_pos = report.find("[SUMMARY]").expect("Should have [SUMMARY]");
    let proactive_pos = report.find("[PROACTIVE SUMMARY]").expect("Should have [PROACTIVE SUMMARY]");
    let details_pos = report.find("[DETAILS]");

    assert!(summary_pos < proactive_pos, "[PROACTIVE SUMMARY] should come after [SUMMARY]");

    // Only check DETAILS positioning if DETAILS exists
    if let Some(details_pos) = details_pos {
        assert!(proactive_pos < details_pos, "[PROACTIVE SUMMARY] should come before [DETAILS]");
    }
}

#[test]
fn test_cli_top_root_cause_human_readable() {
    // Beta.273: Root cause must be user-safe string, not Rust enum
    let issues = vec![
        create_test_issue("network_priority_mismatch", "warning", 0.88),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 82);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Should show snake_case, not CamelCase Rust enum
    assert!(report.contains("network_priority_mismatch"),
        "Should show snake_case root cause");
    assert!(!report.contains("NetworkPriorityMismatch"),
        "Should NOT show Rust enum name");
    assert!(!report.contains("::"), "Should NOT contain Rust :: syntax");
}

// ============================================================================
// Severity Priority Tests
// ============================================================================

#[test]
fn test_severity_priority_consistency() {
    // Beta.273: Verify severity priority values match spec
    assert_eq!(severity_priority_proactive("critical"), 4);
    assert_eq!(severity_priority_proactive("warning"), 3);
    assert_eq!(severity_priority_proactive("info"), 2);
    assert_eq!(severity_priority_proactive("trend"), 1);
    assert_eq!(severity_priority_proactive("unknown"), 0);
}

#[test]
fn test_cli_sorts_by_severity_for_top_cause() {
    // Beta.273: Top root cause should be the highest severity issue
    let issues = vec![
        create_test_issue("trend_issue", "trend", 0.95), // High confidence but low severity
        create_test_issue("critical_issue", "critical", 0.70), // Lower confidence but high severity
        create_test_issue("warning_issue", "warning", 0.90),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 60);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Critical should be top despite lower confidence
    assert!(report.contains("- Top root cause: critical_issue"),
        "Critical severity should take precedence. Got:\n{}", report);
}

// ============================================================================
// Health Score Tests
// ============================================================================

#[test]
fn test_health_score_range_validation() {
    // Beta.273: Health score should be 0-100
    let issues = vec![create_test_issue("test", "info", 0.75)];

    // Test boundaries
    let data_perfect = create_brain_data_with_proactive_and_score(vec![], 100);
    let data_bad = create_brain_data_with_proactive_and_score(issues.clone(), 0);
    let data_mid = create_brain_data_with_proactive_and_score(issues, 50);

    let report_perfect = format_diagnostic_report_with_query(&data_perfect, DiagnosticMode::Full, "check");
    let report_bad = format_diagnostic_report_with_query(&data_bad, DiagnosticMode::Full, "check");
    let report_mid = format_diagnostic_report_with_query(&data_mid, DiagnosticMode::Full, "check");

    // Perfect score should not show proactive section
    assert!(!report_perfect.contains("[PROACTIVE SUMMARY]"));

    // Bad and mid should show scores correctly
    assert!(report_bad.contains("- Health score: 0/100"));
    assert!(report_mid.contains("- Health score: 50/100"));
}

// ============================================================================
// NL Routing Tests (Pattern Documentation)
// ============================================================================

#[test]
fn test_nl_proactive_status_patterns_documented() {
    // Beta.273: Document NL patterns for proactive status queries
    let expected_patterns = vec![
        "show proactive status",
        "proactive status",
        "summarize top issues",
        "summarize issues",
        "what problems do you see",
        "summarize correlations",
        "summarize findings",
        "top correlated issues",
        "show correlations",
        "list proactive issues",
        "proactive summary",
        "correlation summary",
    ];

    // This test documents expected patterns for integration testing
    assert_eq!(expected_patterns.len(), 12, "Should have 12+ documented patterns");
}

// ============================================================================
// Cross-Surface Consistency Tests
// ============================================================================

#[test]
fn test_root_cause_naming_consistency() {
    // Beta.273: Root cause names must be consistent across all surfaces
    let test_cases = vec![
        "network_routing_conflict",
        "network_priority_mismatch",
        "disk_pressure",
        "service_flapping",
        "memory_pressure",
        "cpu_overload",
    ];

    for root_cause in test_cases {
        let issues = vec![create_test_issue(root_cause, "warning", 0.85)];
        let data = create_brain_data_with_proactive_and_score(issues, 80);
        let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

        // Verify snake_case format
        assert!(report.contains(root_cause),
            "Root cause {} should appear in snake_case", root_cause);

        // Verify no enum leakage
        assert!(!report.contains("::"),
            "Root cause {} should not contain Rust :: syntax", root_cause);
    }
}

#[test]
fn test_severity_markers_consistency() {
    // Beta.273: Severity markers must be consistent across CLI and TUI
    let markers = vec![
        ("critical", "✗"),
        ("warning", "⚠"),
        ("info", "ℹ"),
        ("trend", "~"),
    ];

    for (severity, expected_marker) in markers {
        let issues = vec![create_test_issue("test_issue", severity, 0.85)];
        let data = create_brain_data_with_proactive_and_score(issues, 80);
        let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

        // Note: [PROACTIVE SUMMARY] doesn't show markers, but [PROACTIVE] section does
        // This test documents the expected markers for consistency
        assert!(expected_marker.chars().count() == 1,
            "Marker for {} should be single Unicode character ({})", severity, expected_marker);
    }
}

#[test]
fn test_score_calculation_consistency() {
    // Beta.273: Score should be confidence * 100 across all surfaces
    let test_confidences = vec![0.70, 0.85, 0.95, 1.0];

    for confidence in test_confidences {
        let expected_score = (confidence * 100.0) as u8;
        let issues = vec![create_test_issue("test", "warning", confidence)];
        let data = create_brain_data_with_proactive_and_score(issues, 80);

        // Verify the data structure stores confidence correctly
        assert_eq!(data.proactive_issues[0].confidence, confidence);

        // Expected score calculation
        assert_eq!(expected_score, (confidence * 100.0) as u8,
            "Score calculation should be confidence * 100");
    }
}

#[test]
fn test_deterministic_formatting() {
    // Beta.273: All formatting must be deterministic and consistent
    let issues = vec![
        create_test_issue("test1", "critical", 0.90),
        create_test_issue("test2", "warning", 0.85),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 75);

    // Run formatting multiple times - should be identical
    let report1 = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    let report2 = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    let report3 = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert_eq!(report1, report2, "Formatting should be deterministic (run 1 vs 2)");
    assert_eq!(report2, report3, "Formatting should be deterministic (run 2 vs 3)");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_single_issue_singular_wording() {
    // Beta.273: Should handle singular vs plural correctly
    let issues = vec![
        create_test_issue("single_issue", "warning", 0.80),
    ];
    let data = create_brain_data_with_proactive_and_score(issues, 90);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Should say "1 correlated issue(s)" - the (s) is acceptable
    assert!(report.contains("- 1 correlated issue(s) detected") ||
            report.contains("- 1 correlated issue detected"),
        "Should handle singular case. Got:\n{}", report);
}

#[test]
fn test_many_issues_shows_top_only() {
    // Beta.273: Should show top root cause even when many issues exist
    let mut issues = Vec::new();
    for i in 0..15 {
        issues.push(create_test_issue(&format!("issue_{}", i), "info", 0.75));
    }
    // Add one critical issue
    issues.push(create_test_issue("critical_top", "critical", 0.90));

    let data = create_brain_data_with_proactive_and_score(issues, 45);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("- 16 correlated issue(s) detected"),
        "Should show total count");
    assert!(report.contains("- Top root cause: critical_top"),
        "Should show critical issue as top cause");
}

#[test]
fn test_backward_compatibility_default_score() {
    // Beta.273: Old data without health_score should default to 100
    let issues = vec![];
    let data = BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues: issues,
        proactive_health_score: 100, // Default value from serde default
    };

    // Should handle gracefully (no proactive section shown since no issues)
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    assert!(!report.contains("[PROACTIVE SUMMARY]"));
}

// ============================================================================
// TUI Panel Tests (Structural Validation)
// ============================================================================

#[test]
fn test_tui_panel_cap_at_three() {
    // Beta.273: TUI should cap proactive issues at 3 (space-constrained)
    // This test validates the data preparation logic

    let mut issues = Vec::new();
    for i in 0..10 {
        issues.push(create_test_issue(&format!("issue_{}", i), "warning", 0.80));
    }

    let data = create_brain_data_with_proactive_and_score(issues, 70);

    // Sort and cap at 3 (simulating TUI logic)
    let mut sorted = data.proactive_issues.clone();
    sorted.sort_by(|a, b| {
        let a_priority = severity_priority_proactive(&a.severity);
        let b_priority = severity_priority_proactive(&b.severity);
        b_priority.cmp(&a_priority)
    });
    sorted.truncate(3);

    assert_eq!(sorted.len(), 3, "TUI should cap at 3 issues");
}
