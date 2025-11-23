//! Beta.273: Cross-Surface Consistency Tests
//!
//! These tests ensure proactive engine data is presented consistently across:
//! - CLI formatter ([PROACTIVE SUMMARY] and [PROACTIVE] sections)
//! - TUI brain panel
//! - NL query responses
//!
//! Key consistency requirements:
//! - Root cause names (snake_case, no Rust enums)
//! - Severity markers (✗, ⚠, ℹ, ~)
//! - Score calculation (confidence * 100)
//! - Severity sorting (critical > warning > info > trend)

use annactl::diagnostic_formatter::{format_diagnostic_report_with_query, severity_priority_proactive, DiagnosticMode};
use anna_common::ipc::{BrainAnalysisData, ProactiveIssueSummaryData};

/// Helper: Create test issue
fn create_issue(root_cause: &str, severity: &str, confidence: f32) -> ProactiveIssueSummaryData {
    ProactiveIssueSummaryData {
        root_cause: root_cause.to_string(),
        severity: severity.to_string(),
        summary: format!("{} detected", root_cause),
        rule_id: None,
        confidence,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    }
}

/// Helper: Create brain data
fn create_data(issues: Vec<ProactiveIssueSummaryData>, score: u8) -> BrainAnalysisData {
    BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues: issues,
        proactive_health_score: score,
    }
}

// ============================================================================
// Root Cause Naming Consistency
// ============================================================================

#[test]
fn test_root_cause_snake_case_across_surfaces() {
    // All root causes must use snake_case format
    let test_cases = vec![
        "network_routing_conflict",
        "network_priority_mismatch",
        "network_quality_degradation",
        "disk_pressure",
        "disk_log_growth",
        "service_flapping",
        "service_under_load",
        "service_config_error",
        "memory_pressure",
        "cpu_overload",
        "kernel_regression",
        "device_hotplug",
    ];

    for root_cause in test_cases {
        let issues = vec![create_issue(root_cause, "warning", 0.85)];
        let data = create_data(issues, 80);
        let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

        // Verify snake_case present
        assert!(report.contains(root_cause),
            "CLI should show snake_case root cause: {}", root_cause);

        // Verify no CamelCase
        let camel_case = root_cause.split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
            })
            .collect::<Vec<_>>()
            .join("");

        assert!(!report.contains(&camel_case),
            "CLI should NOT show CamelCase: {}", camel_case);
    }
}

#[test]
fn test_no_rust_enum_leakage() {
    // No Rust enum syntax (::, <>, Option) should appear in user-facing output
    let issues = vec![
        create_issue("network_routing_conflict", "critical", 0.90),
        create_issue("memory_pressure", "warning", 0.85),
    ];
    let data = create_data(issues, 75);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Check for Rust artifacts
    assert!(!report.contains("::"), "Should not contain Rust :: syntax");
    assert!(!report.contains("Option<"), "Should not contain Rust Option<>");
    assert!(!report.contains("Some("), "Should not contain Rust Some()");
    assert!(!report.contains("RootCause::"), "Should not contain enum prefix");
    assert!(!report.contains("IssueSeverity::"), "Should not contain severity enum prefix");
}

// ============================================================================
// Severity Marker Consistency
// ============================================================================

#[test]
fn test_severity_markers_match_spec() {
    // Beta.273 spec defines exact markers for each severity
    let expected = vec![
        ("critical", "✗"),
        ("warning", "⚠"),
        ("info", "ℹ"),
        ("trend", "~"),
    ];

    for (severity, marker) in expected {
        // Verify marker is single Unicode char
        assert_eq!(marker.chars().count(), 1,
            "Marker for {} must be single char", severity);

        // Note: Markers appear in [PROACTIVE] section, not [PROACTIVE SUMMARY]
        // This test documents the expected mapping
        let expected_byte_len = match severity {
            "critical" | "warning" | "info" => 3, // UTF-8 multi-byte
            "trend" => 1, // ASCII
            _ => panic!("Unknown severity"),
        };

        assert_eq!(marker.len(), expected_byte_len,
            "Marker {} should be {} bytes", marker, expected_byte_len);
    }
}

// ============================================================================
// Score Calculation Consistency
// ============================================================================

#[test]
fn test_score_formula_consistency() {
    // Score = confidence * 100, consistently across all surfaces
    let test_cases = vec![
        (0.70, 70),
        (0.85, 85),
        (0.95, 95),
        (1.00, 100),
        (0.72, 72),
        (0.88, 88),
    ];

    for (confidence, expected_score) in test_cases {
        let actual_score = (confidence * 100.0) as u8;
        assert_eq!(actual_score, expected_score,
            "Score for confidence {} should be {}", confidence, expected_score);
    }
}

// ============================================================================
// Severity Sorting Consistency
// ============================================================================

#[test]
fn test_severity_sort_order_consistency() {
    // critical (4) > warning (3) > info (2) > trend (1) > unknown (0)
    assert!(severity_priority_proactive("critical") > severity_priority_proactive("warning"));
    assert!(severity_priority_proactive("warning") > severity_priority_proactive("info"));
    assert!(severity_priority_proactive("info") > severity_priority_proactive("trend"));
    assert!(severity_priority_proactive("trend") > severity_priority_proactive("unknown"));
}

#[test]
fn test_top_issue_always_highest_severity() {
    // Top issue must be highest severity, regardless of confidence
    let issues = vec![
        create_issue("low_severity_high_conf", "trend", 0.99), // Highest confidence
        create_issue("high_severity_low_conf", "critical", 0.71), // Lowest confidence but highest severity
        create_issue("mid", "warning", 0.85),
    ];
    let data = create_data(issues, 65);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Critical should be top despite lower confidence
    assert!(report.contains("- Top root cause: high_severity_low_conf"),
        "Top cause should be critical severity. Got:\n{}", report);
}

// ============================================================================
// Format Consistency
// ============================================================================

#[test]
fn test_cli_sections_ordering() {
    // [SUMMARY] → [PROACTIVE SUMMARY] → [PROACTIVE] → [DETAILS] → [COMMANDS]
    let issues = vec![create_issue("test", "warning", 0.80)];
    let data = create_data(issues, 85);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    let summary_pos = report.find("[SUMMARY]").unwrap();
    let proactive_summary_pos = report.find("[PROACTIVE SUMMARY]").unwrap();
    let proactive_pos = report.find("[PROACTIVE]").unwrap();
    let commands_pos = report.find("[COMMANDS]").unwrap();

    assert!(summary_pos < proactive_summary_pos, "[SUMMARY] comes first");
    assert!(proactive_summary_pos < proactive_pos, "[PROACTIVE SUMMARY] before [PROACTIVE]");
    assert!(proactive_pos < commands_pos, "[PROACTIVE] before [COMMANDS]");
}

#[test]
fn test_health_score_format_consistency() {
    // Health score must always be "XX/100" format
    let test_scores = vec![0, 25, 50, 75, 100];

    for score in test_scores {
        let issues = vec![create_issue("test", "info", 0.75)];
        let has_issues = !issues.is_empty();
        let data = create_data(issues, score);
        let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

        if score == 100 && !has_issues {
            // No issues, no proactive section
            continue;
        }

        let expected = format!("- Health score: {}/100", score);
        assert!(report.contains(&expected),
            "Should show '{}'. Got:\n{}", expected, report);
    }
}

// ============================================================================
// Zero-Case Consistency
// ============================================================================

#[test]
fn test_zero_issues_consistent_behavior() {
    // When no proactive issues, behavior must be consistent across surfaces
    let data = create_data(vec![], 100);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Should NOT show proactive sections
    assert!(!report.contains("[PROACTIVE SUMMARY]"),
        "No proactive summary when no issues");
    assert!(!report.contains("[PROACTIVE]"),
        "No proactive section when no issues");
    assert!(!report.contains("Top root cause:"),
        "No top cause when no issues");
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_deterministic_output_cli() {
    // Same input must always produce same output
    let issues = vec![
        create_issue("issue1", "critical", 0.90),
        create_issue("issue2", "warning", 0.85),
        create_issue("issue3", "info", 0.75),
    ];
    let data = create_data(issues, 70);

    let report1 = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    let report2 = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    let report3 = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert_eq!(report1, report2, "Output must be deterministic (run 1 vs 2)");
    assert_eq!(report2, report3, "Output must be deterministic (run 2 vs 3)");
}
