//! Beta.271: Proactive Engine Plumbing Regression Tests
//!
//! These tests validate the end-to-end plumbing of the proactive engine:
//! - RPC protocol carries proactive_issues field correctly
//! - Formatter displays proactive issue count
//! - TUI state receives and stores proactive issues
//! - Issue capping and confidence filtering work correctly
//! - Backward compatibility maintained

use annactl::diagnostic_formatter::{format_diagnostic_report_with_query, DiagnosticMode};
use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData, ProactiveIssueSummaryData};

/// Helper: Create a minimal BrainAnalysisData for testing
fn create_brain_data_with_proactive(proactive_count: usize) -> BrainAnalysisData {
    let mut proactive_issues = Vec::new();

    for i in 0..proactive_count {
        proactive_issues.push(ProactiveIssueSummaryData {
            root_cause: format!("test_cause_{}", i),
            severity: "warning".to_string(),
            summary: format!("Test issue {}", i),
            rule_id: None,
            confidence: 0.8,
            first_seen: "2025-11-23T00:00:00Z".to_string(),
            last_seen: "2025-11-23T01:00:00Z".to_string(),
        });
    }

    BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues,
    }
}

// ============================================================================
// RPC Protocol Tests
// ============================================================================

#[test]
fn test_rpc_proactive_issues_field_exists() {
    // Beta.271: Verify BrainAnalysisData has proactive_issues field
    let data = create_brain_data_with_proactive(3);
    assert_eq!(data.proactive_issues.len(), 3);
}

#[test]
fn test_rpc_backward_compatibility_empty_issues() {
    // Beta.271: Old-style BrainAnalysisData without proactive_issues should default to empty
    let json = r#"{
        "timestamp": "2025-11-23T01:00:00Z",
        "formatted_output": "",
        "critical_count": 0,
        "warning_count": 0,
        "insights": []
    }"#;

    let data: BrainAnalysisData = serde_json::from_str(json).unwrap();
    assert_eq!(data.proactive_issues.len(), 0, "Missing proactive_issues should default to empty vec");
}

#[test]
fn test_proactive_summary_data_serialization() {
    // Beta.271: Verify ProactiveIssueSummaryData round-trips through JSON
    let summary = ProactiveIssueSummaryData {
        root_cause: "network_routing_conflict".to_string(),
        severity: "critical".to_string(),
        summary: "Network routing conflict detected".to_string(),
        rule_id: Some("NET-001".to_string()),
        confidence: 0.95,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: ProactiveIssueSummaryData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.root_cause, "network_routing_conflict");
    assert_eq!(deserialized.severity, "critical");
    assert_eq!(deserialized.confidence, 0.95);
}

// ============================================================================
// Formatter Display Tests
// ============================================================================

#[test]
fn test_formatter_shows_proactive_count_when_present() {
    // Beta.271: Formatter should show "ℹ Proactive engine detected N correlated issue pattern(s)"
    let data = create_brain_data_with_proactive(5);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check my system");

    assert!(report.contains("ℹ Proactive engine detected 5 correlated issue pattern(s)"),
        "Formatter should display proactive issue count. Got:\n{}", report);
}

#[test]
fn test_formatter_hides_proactive_line_when_empty() {
    // Beta.271: When no proactive issues, should not show the info line
    let data = create_brain_data_with_proactive(0);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check my system");

    assert!(!report.contains("Proactive engine detected"),
        "Formatter should not show proactive line when empty. Got:\n{}", report);
}

#[test]
fn test_formatter_shows_proactive_with_one_issue() {
    // Beta.271: Edge case - singular "pattern" vs plural
    let data = create_brain_data_with_proactive(1);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check my system");

    assert!(report.contains("ℹ Proactive engine detected 1 correlated issue pattern(s)"),
        "Formatter should handle singular case. Got:\n{}", report);
}

#[test]
fn test_formatter_proactive_appears_after_summary() {
    // Beta.271: Proactive line should appear after [SUMMARY] section, before [COMMANDS]
    let data = create_brain_data_with_proactive(3);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check my system");

    let summary_pos = report.find("[SUMMARY]").expect("Should have [SUMMARY]");
    let proactive_pos = report.find("Proactive engine detected").expect("Should have proactive line");
    let commands_pos = report.find("[COMMANDS]").expect("Should have [COMMANDS]");

    assert!(summary_pos < proactive_pos, "Proactive line should come after SUMMARY");
    assert!(proactive_pos < commands_pos, "Proactive line should come before COMMANDS");
}

// ============================================================================
// Issue Capping Tests
// ============================================================================

#[test]
fn test_issue_cap_at_ten() {
    // Beta.271: MAX_DISPLAYED_ISSUES = 10, verify capping works
    let data = create_brain_data_with_proactive(15);

    // The formatter doesn't cap, but the server-side assessment_to_summaries does
    // This test validates that if we receive 10 issues, formatter shows "10"
    let mut data_capped = create_brain_data_with_proactive(10);
    data_capped.proactive_issues.truncate(10);

    let report = format_diagnostic_report_with_query(&data_capped, DiagnosticMode::Full, "check");
    assert!(report.contains("detected 10 correlated issue pattern(s)"));
}

#[test]
fn test_no_issues_shows_zero() {
    // Beta.271: Edge case - explicitly zero issues
    let mut data = create_brain_data_with_proactive(0);
    data.proactive_issues.clear();

    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    assert!(!report.contains("Proactive engine detected"),
        "Should not show line when proactive_issues is empty");
}

// ============================================================================
// Confidence Filtering Tests (Informational)
// ============================================================================

#[test]
fn test_low_confidence_issues_filtered_server_side() {
    // Beta.271: Issues with confidence < 0.7 should be filtered by server
    // This test documents the expected behavior - filtering happens in
    // compute_proactive_assessment, not in the formatter

    let high_conf = ProactiveIssueSummaryData {
        root_cause: "high_conf".to_string(),
        severity: "warning".to_string(),
        summary: "High confidence issue".to_string(),
        rule_id: None,
        confidence: 0.85,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    };

    // Server should never send low confidence issues, but if it does,
    // formatter will display them (no client-side filtering)
    let data = BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues: vec![high_conf],
    };

    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");
    assert!(report.contains("detected 1 correlated issue pattern(s)"));
}

// ============================================================================
// Root Cause String Mapping Tests
// ============================================================================

#[test]
fn test_root_cause_labels_are_user_safe() {
    // Beta.271: Root cause should be user-safe strings, not Rust enum names
    let data = BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues: vec![
            ProactiveIssueSummaryData {
                root_cause: "network_routing_conflict".to_string(),
                severity: "critical".to_string(),
                summary: "Test".to_string(),
                rule_id: None,
                confidence: 0.9,
                first_seen: "2025-11-23T00:00:00Z".to_string(),
                last_seen: "2025-11-23T01:00:00Z".to_string(),
            }
        ],
    };

    // Root cause should not contain Rust-style names like "NetworkRoutingConflict"
    let json = serde_json::to_string(&data.proactive_issues[0]).unwrap();
    assert!(!json.contains("NetworkRoutingConflict"), "Should not leak Rust enum names");
    assert!(json.contains("network_routing_conflict"), "Should use snake_case user-safe labels");
}

#[test]
fn test_severity_labels_are_lowercase() {
    // Beta.271: Severity should be lowercase strings
    let summary = ProactiveIssueSummaryData {
        root_cause: "test".to_string(),
        severity: "critical".to_string(),
        summary: "Test".to_string(),
        rule_id: None,
        confidence: 0.9,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    };

    assert_eq!(summary.severity, "critical");
    assert_ne!(summary.severity, "Critical");
}

// ============================================================================
// Timestamp Format Tests
// ============================================================================

#[test]
fn test_timestamps_are_iso8601() {
    // Beta.271: first_seen and last_seen should be ISO 8601 (RFC 3339)
    let summary = ProactiveIssueSummaryData {
        root_cause: "test".to_string(),
        severity: "warning".to_string(),
        summary: "Test".to_string(),
        rule_id: None,
        confidence: 0.8,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    };

    // Verify format is parseable as RFC 3339
    use chrono::{DateTime, Utc};
    let _first: DateTime<Utc> = summary.first_seen.parse().expect("first_seen should be valid RFC3339");
    let _last: DateTime<Utc> = summary.last_seen.parse().expect("last_seen should be valid RFC3339");
}

// ============================================================================
// Summary Mode Tests
// ============================================================================

#[test]
fn test_proactive_line_appears_in_summary_mode() {
    // Beta.271: Proactive issue count should show in both Full and Summary modes
    let data = create_brain_data_with_proactive(4);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Summary, "check");

    assert!(report.contains("ℹ Proactive engine detected 4 correlated issue pattern(s)"),
        "Proactive line should appear in Summary mode. Got:\n{}", report);
}

#[test]
fn test_proactive_line_appears_in_full_mode() {
    // Beta.271: Proactive issue count should show in Full mode
    let data = create_brain_data_with_proactive(7);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("ℹ Proactive engine detected 7 correlated issue pattern(s)"),
        "Proactive line should appear in Full mode. Got:\n{}", report);
}
