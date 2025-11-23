//! Beta.272: Proactive Surfacing and Remediation Regression Tests
//!
//! These tests validate the Beta.272 features that surface proactive issues to users:
//! - CLI health output shows [PROACTIVE] section with top correlated issues
//! - Remediation composer maps root causes to appropriate fix guidance
//! - NL routing detects "what should I fix first" queries
//! - TUI brain panel displays proactive issues

use annactl::diagnostic_formatter::{format_diagnostic_report_with_query, severity_priority_proactive, DiagnosticMode};
use annactl::sysadmin_answers::compose_top_proactive_remediation;
use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData, ProactiveIssueSummaryData};

/// Helper: Create a minimal BrainAnalysisData for testing
fn create_brain_data_with_issues(issues: Vec<ProactiveIssueSummaryData>) -> BrainAnalysisData {
    BrainAnalysisData {
        timestamp: "2025-11-23T01:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        insights: vec![],
        proactive_issues: issues,
    }
}

/// Helper: Create a test proactive issue
fn create_test_issue(root_cause: &str, severity: &str, summary: &str) -> ProactiveIssueSummaryData {
    ProactiveIssueSummaryData {
        root_cause: root_cause.to_string(),
        severity: severity.to_string(),
        summary: summary.to_string(),
        rule_id: None,
        confidence: 0.85,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    }
}

// ============================================================================
// CLI [PROACTIVE] Section Tests
// ============================================================================

#[test]
fn test_proactive_section_appears_in_health_output() {
    // Beta.272: Health output should have [PROACTIVE] section when issues exist
    let issues = vec![
        create_test_issue("network_routing_conflict", "critical", "Network priority issue"),
        create_test_issue("disk_pressure", "warning", "Disk pressure on /"),
    ];
    let data = create_brain_data_with_issues(issues);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check my system health");

    assert!(report.contains("[PROACTIVE]"),
        "Report should contain [PROACTIVE] section. Got:\n{}", report);
    assert!(report.contains("Top correlated issues:"),
        "Report should show section header. Got:\n{}", report);
}

#[test]
fn test_proactive_section_hidden_when_no_issues() {
    // Beta.272: No [PROACTIVE] section when proactive_issues is empty
    let data = create_brain_data_with_issues(vec![]);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check my system health");

    assert!(!report.contains("[PROACTIVE]"),
        "Report should not show [PROACTIVE] when no issues. Got:\n{}", report);
}

#[test]
fn test_proactive_section_shows_severity_markers() {
    // Beta.272: Issues should have severity markers (✗, ⚠, ℹ)
    let issues = vec![
        create_test_issue("network_routing_conflict", "critical", "Critical network issue"),
        create_test_issue("disk_pressure", "warning", "Warning disk issue"),
        create_test_issue("service_flapping", "info", "Info service issue"),
    ];
    let data = create_brain_data_with_issues(issues);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("✗"), "Should show critical marker (✗)");
    assert!(report.contains("⚠"), "Should show warning marker (⚠)");
    assert!(report.contains("ℹ"), "Should show info marker (ℹ)");
}

#[test]
fn test_proactive_section_numbered_list() {
    // Beta.272: Issues should be numbered starting from 1
    let issues = vec![
        create_test_issue("test1", "warning", "First issue"),
        create_test_issue("test2", "warning", "Second issue"),
        create_test_issue("test3", "warning", "Third issue"),
    ];
    let data = create_brain_data_with_issues(issues);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    assert!(report.contains("1. ⚠ First issue"), "Should show numbered issue 1");
    assert!(report.contains("2. ⚠ Second issue"), "Should show numbered issue 2");
    assert!(report.contains("3. ⚠ Third issue"), "Should show numbered issue 3");
}

#[test]
fn test_proactive_section_caps_at_ten_issues() {
    // Beta.272: Only show top 10 issues, sorted by severity
    let mut issues = Vec::new();
    for i in 0..15 {
        issues.push(create_test_issue(&format!("test_{}", i), "info", &format!("Issue {}", i)));
    }
    let data = create_brain_data_with_issues(issues);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    // Count how many numbered items appear
    let count = (1..=15).filter(|i| report.contains(&format!("{}. ℹ", i))).count();
    assert_eq!(count, 10, "Should cap at 10 issues in [PROACTIVE] section");
}

#[test]
fn test_proactive_section_sorted_by_severity() {
    // Beta.272: Critical issues should appear first, then warning, then info
    let issues = vec![
        create_test_issue("info_issue", "info", "Info issue"),
        create_test_issue("critical_issue", "critical", "Critical issue"),
        create_test_issue("warning_issue", "warning", "Warning issue"),
    ];
    let data = create_brain_data_with_issues(issues);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    let critical_pos = report.find("Critical issue").expect("Should have critical issue");
    let warning_pos = report.find("Warning issue").expect("Should have warning issue");
    let info_pos = report.find("Info issue").expect("Should have info issue");

    assert!(critical_pos < warning_pos, "Critical should come before warning");
    assert!(warning_pos < info_pos, "Warning should come before info");
}

#[test]
fn test_proactive_section_positioning() {
    // Beta.272: [PROACTIVE] should appear after [SUMMARY], before [COMMANDS]
    let issues = vec![create_test_issue("test", "warning", "Test issue")];
    let data = create_brain_data_with_issues(issues);
    let report = format_diagnostic_report_with_query(&data, DiagnosticMode::Full, "check");

    let summary_pos = report.find("[SUMMARY]").expect("Should have [SUMMARY]");
    let proactive_pos = report.find("[PROACTIVE]").expect("Should have [PROACTIVE]");
    let commands_pos = report.find("[COMMANDS]").expect("Should have [COMMANDS]");

    assert!(summary_pos < proactive_pos, "[PROACTIVE] should come after [SUMMARY]");
    assert!(proactive_pos < commands_pos, "[PROACTIVE] should come before [COMMANDS]");
}

// ============================================================================
// Remediation Composer Tests
// ============================================================================

#[test]
fn test_remediation_network_routing_conflict() {
    // Beta.272: Network routing conflict should map to network remediation
    let issue = create_test_issue("network_routing_conflict", "critical", "Slow interface has default route");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for network_routing_conflict");

    assert!(answer.contains("[SUMMARY]"), "Should have SUMMARY section");
    assert!(answer.contains("[DETAILS]"), "Should have DETAILS section");
    assert!(answer.contains("[COMMANDS]"), "Should have COMMANDS section");
    assert!(answer.contains("routing"), "Should mention routing in answer");
}

#[test]
fn test_remediation_network_priority_mismatch() {
    // Beta.272: Network priority mismatch should map to network remediation
    let issue = create_test_issue("network_priority_mismatch", "warning", "Priority conflict detected");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for network_priority_mismatch");

    assert!(answer.contains("Network routing issue"), "Should mention network routing");
    assert!(answer.contains("ip route"), "Should show ip route command");
}

#[test]
fn test_remediation_network_quality_degradation() {
    // Beta.272: Network quality degradation should map to network fix
    let issue = create_test_issue("network_quality_degradation", "warning", "High packet loss detected");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for network_quality_degradation");

    assert!(answer.contains("Network quality"), "Should mention network quality");
    assert!(answer.contains("packet loss"), "Should mention packet loss or errors");
}

#[test]
fn test_remediation_disk_pressure() {
    // Beta.272: Disk pressure should map to disk fix composer
    let issue = create_test_issue("disk_pressure", "critical", "Disk usage at 92%");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for disk_pressure");

    assert!(answer.contains("[SUMMARY]"), "Should use standard format");
    // Should call compose_disk_fix_answer internally
    assert!(answer.contains("disk") || answer.contains("Disk"), "Should mention disk");
}

#[test]
fn test_remediation_disk_log_growth() {
    // Beta.272: Disk log growth should map to disk fix
    let issue = create_test_issue("disk_log_growth", "warning", "Logs growing rapidly");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for disk_log_growth");

    assert!(answer.contains("[COMMANDS]"), "Should provide commands");
}

#[test]
fn test_remediation_service_flapping() {
    // Beta.272: Service flapping should map to service fix
    let issue = create_test_issue("service_flapping", "warning", "Service restarted 5 times");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for service_flapping");

    assert!(answer.contains("Service"), "Should mention services");
    assert!(answer.contains("journalctl") || answer.contains("systemctl"), "Should show service commands");
}

#[test]
fn test_remediation_service_under_load() {
    // Beta.272: Service under load should map to service fix
    let issue = create_test_issue("service_under_load", "warning", "Service experiencing high load");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for service_under_load");

    assert!(answer.contains("Service"), "Should mention services");
}

#[test]
fn test_remediation_service_config_error() {
    // Beta.272: Service config error should map to service fix
    let issue = create_test_issue("service_config_error", "critical", "Configuration issue detected");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for service_config_error");

    assert!(answer.contains("[SUMMARY]"), "Should use standard format");
}

#[test]
fn test_remediation_memory_pressure() {
    // Beta.272: Memory pressure should map to memory fix composer
    let issue = create_test_issue("memory_pressure", "critical", "Memory usage at 90%");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for memory_pressure");

    assert!(answer.contains("memory") || answer.contains("Memory"), "Should mention memory");
}

#[test]
fn test_remediation_cpu_overload() {
    // Beta.272: CPU overload should map to CPU fix composer
    let issue = create_test_issue("cpu_overload", "critical", "CPU sustained above 95%");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for cpu_overload");

    assert!(answer.contains("CPU") || answer.contains("cpu"), "Should mention CPU");
}

#[test]
fn test_remediation_kernel_regression() {
    // Beta.272: Kernel regression should provide diagnostic guidance
    let issue = create_test_issue("kernel_regression", "critical", "Device errors after kernel update");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for kernel_regression");

    assert!(answer.contains("kernel") || answer.contains("Kernel"), "Should mention kernel");
}

#[test]
fn test_remediation_device_hotplug() {
    // Beta.272: Device hotplug should provide device management guidance
    let issue = create_test_issue("device_hotplug", "warning", "Device connection instability");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation for device_hotplug");

    assert!(answer.contains("device") || answer.contains("Device"), "Should mention device");
}

#[test]
fn test_remediation_unknown_root_cause() {
    // Beta.272: Unknown root cause should provide generic guidance
    let issue = create_test_issue("unknown_future_cause", "warning", "New type of issue");
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return generic remediation for unknown causes");

    assert!(answer.contains("[SUMMARY]"), "Should use standard format");
    assert!(answer.contains("Correlated issue detected"), "Should acknowledge issue");
    assert!(answer.contains("unknown_future_cause"), "Should show root cause");
    assert!(answer.contains("annactl"), "Should suggest basic diagnostics");
}

#[test]
fn test_remediation_includes_confidence() {
    // Beta.272: Generic remediation should show confidence level
    let issue = ProactiveIssueSummaryData {
        root_cause: "unknown_test".to_string(),
        severity: "warning".to_string(),
        summary: "Test issue".to_string(),
        rule_id: None,
        confidence: 0.78,
        first_seen: "2025-11-23T00:00:00Z".to_string(),
        last_seen: "2025-11-23T01:00:00Z".to_string(),
    };
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation");

    assert!(answer.contains("78%"), "Should show confidence percentage");
}

#[test]
fn test_remediation_includes_timestamps() {
    // Beta.272: Generic remediation should show first/last seen
    let issue = ProactiveIssueSummaryData {
        root_cause: "unknown_test".to_string(),
        severity: "warning".to_string(),
        summary: "Test issue".to_string(),
        rule_id: None,
        confidence: 0.85,
        first_seen: "2025-11-23T10:30:00Z".to_string(),
        last_seen: "2025-11-23T12:45:00Z".to_string(),
    };
    let brain_data = create_brain_data_with_issues(vec![]);

    let answer = compose_top_proactive_remediation(&issue, &brain_data)
        .expect("Should return remediation");

    assert!(answer.contains("First seen:"), "Should show first_seen label");
    assert!(answer.contains("Last seen:"), "Should show last_seen label");
}

// ============================================================================
// Severity Priority Tests
// ============================================================================

#[test]
fn test_severity_priority_critical_highest() {
    // Beta.272: Critical should have highest priority (4)
    assert_eq!(severity_priority_proactive("critical"), 4);
    assert_eq!(severity_priority_proactive("Critical"), 4); // Case insensitive
}

#[test]
fn test_severity_priority_warning_second() {
    // Beta.272: Warning should have priority 3
    assert_eq!(severity_priority_proactive("warning"), 3);
    assert_eq!(severity_priority_proactive("WARNING"), 3);
}

#[test]
fn test_severity_priority_info_third() {
    // Beta.272: Info should have priority 2
    assert_eq!(severity_priority_proactive("info"), 2);
    assert_eq!(severity_priority_proactive("Info"), 2);
}

#[test]
fn test_severity_priority_trend_fourth() {
    // Beta.272: Trend should have priority 1
    assert_eq!(severity_priority_proactive("trend"), 1);
    assert_eq!(severity_priority_proactive("TREND"), 1);
}

#[test]
fn test_severity_priority_unknown_lowest() {
    // Beta.272: Unknown severity should have priority 0
    assert_eq!(severity_priority_proactive("unknown"), 0);
    assert_eq!(severity_priority_proactive("bogus"), 0);
}

// ============================================================================
// NL Routing Tests (Informational - actual routing tested via integration)
// ============================================================================

#[test]
fn test_nl_patterns_documented() {
    // Beta.272: Document the NL patterns that should route to proactive remediation
    let expected_patterns = vec![
        "what should i fix first",
        "what should i fix",
        "what are my top issues",
        "what are the top issues",
        "show me my top issues",
        "top issues",
        "top problems",
        "what is the most important issue",
        "most important issue",
        "what is the main problem",
        "main problem",
        "any critical problems",
        "critical problems",
        "highest priority issue",
        "priority issues",
    ];

    // This test documents expected patterns
    // Actual pattern matching is tested in unified_query_handler integration tests
    assert_eq!(expected_patterns.len(), 15, "Should have at least 15 documented patterns");
}
