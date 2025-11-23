//! Beta.278: Sysadmin Report v1 - Regression Test Suite
//!
//! Tests for the comprehensive sysadmin briefing feature that combines:
//! - Health summary and diagnostic insights
//! - Daily snapshot (session deltas)
//! - Proactive correlated issues
//! - Key domain highlights (services, disk, network, resources)
//!
//! Test Coverage:
//! - Routing tests (8-10 tests): NL pattern detection
//! - Content tests (10-15 tests): Report composition with mock data
//! - Formatting tests (5-7 tests): Canonical format validation
//!
//! Total: ~28 tests

use annactl::unified_query_handler::{is_sysadmin_report_query, normalize_query_for_intent};
use annactl::sysadmin_answers::compose_sysadmin_report_answer;
use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData, ProactiveIssueSummaryData};

// ===========================================================================
// SECTION 1: ROUTING TESTS (8 tests)
// ===========================================================================

#[test]
fn test_routing_sysadmin_report_exact_phrases() {
    // Should match: core sysadmin report phrases
    assert!(is_sysadmin_report_query("sysadmin report"));
    assert!(is_sysadmin_report_query("full sysadmin report"));
    assert!(is_sysadmin_report_query("sysadmin briefing"));
    assert!(is_sysadmin_report_query("system admin report"));
}

#[test]
fn test_routing_full_system_report() {
    // Should match: full system report variants
    assert!(is_sysadmin_report_query("full system report"));
    assert!(is_sysadmin_report_query("complete system report"));
    assert!(is_sysadmin_report_query("full status report"));
    assert!(is_sysadmin_report_query("comprehensive system report"));
}

#[test]
fn test_routing_overall_situation() {
    // Should match: overall/summary phrasing
    assert!(is_sysadmin_report_query("overall situation on this system"));
    assert!(is_sysadmin_report_query("current situation of my machine"));
    assert!(is_sysadmin_report_query("system situation"));
}

#[test]
fn test_routing_summarize_variants() {
    // Should match: summarize phrasing
    assert!(is_sysadmin_report_query("summarize the system"));
    assert!(is_sysadmin_report_query("summarize my machine"));
    assert!(is_sysadmin_report_query("summarize the state of my system"));
}

#[test]
fn test_routing_give_me_imperatives() {
    // Should match: "give me" imperatives
    assert!(is_sysadmin_report_query("give me a full report on this system"));
    assert!(is_sysadmin_report_query("give me a system report"));
    assert!(is_sysadmin_report_query("give me the full picture"));
    assert!(is_sysadmin_report_query("give me everything about my machine"));
}

#[test]
fn test_routing_show_me_imperatives() {
    // Should match: "show me" imperatives
    assert!(is_sysadmin_report_query("show me a full system report"));
    assert!(is_sysadmin_report_query("show me the complete picture"));
    assert!(is_sysadmin_report_query("show me everything"));
}

#[test]
fn test_routing_overview_phrases() {
    // Should match: overview phrases
    assert!(is_sysadmin_report_query("overview of this system"));
    assert!(is_sysadmin_report_query("system overview"));
    assert!(is_sysadmin_report_query("complete overview of my machine"));
}

#[test]
fn test_routing_should_not_match() {
    // Should NOT match: these go to other routes

    // Pure health checks → diagnostic
    assert!(!is_sysadmin_report_query("check my system health"));
    assert!(!is_sysadmin_report_query("is my system healthy"));

    // Domain-specific → sysadmin domain answers
    assert!(!is_sysadmin_report_query("check my network"));
    assert!(!is_sysadmin_report_query("check my disk"));

    // Educational → conversational
    assert!(!is_sysadmin_report_query("what is a healthy system"));
    assert!(!is_sysadmin_report_query("explain system health"));

    // Lighter status queries → system_report (existing)
    assert!(!is_sysadmin_report_query("status"));
    assert!(!is_sysadmin_report_query("show me status"));
}

// ===========================================================================
// SECTION 2: CONTENT TESTS (12 tests)
// ===========================================================================

fn create_healthy_brain() -> BrainAnalysisData {
    BrainAnalysisData {
        timestamp: "2025-11-23T12:00:00Z".to_string(),
        insights: vec![],
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        proactive_issues: vec![],
        proactive_health_score: 100,
    }
}

fn create_degraded_brain() -> BrainAnalysisData {
    BrainAnalysisData {
        timestamp: "2025-11-23T12:00:00Z".to_string(),
        insights: vec![
            DiagnosticInsightData {
                rule_id: "check_systemd".to_string(),
                summary: "1 failed service detected (docker.service)".to_string(),
                details: "docker.service is in failed state".to_string(),
                evidence: "systemctl --failed".to_string(),
                severity: "critical".to_string(),
                commands: vec!["systemctl status docker.service".to_string()],
                citations: vec![],
            },
            DiagnosticInsightData {
                rule_id: "check_disk".to_string(),
                summary: "Root partition at 85% capacity".to_string(),
                details: "/ is at 85% usage".to_string(),
                evidence: "df -h".to_string(),
                severity: "warning".to_string(),
                commands: vec!["df -h /".to_string()],
                citations: vec![],
            },
        ],
        formatted_output: String::new(),
        critical_count: 1,
        warning_count: 1,
        proactive_issues: vec![],
        proactive_health_score: 75,
    }
}

#[test]
fn test_content_healthy_system_all_clear() {
    let brain = create_healthy_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 100);

    // Should say "all clear"
    assert!(report.contains("[SUMMARY]"));
    assert!(report.contains("all clear"));
    assert!(report.contains("no critical issues"));

    // Should confirm no issues
    assert!(report.contains("[HEALTH]"));
    assert!(report.contains("All diagnostic checks passed") ||
            report.contains("no issues") || report.contains("All key domains"));

    // Should show no proactive issues
    assert!(report.contains("[PROACTIVE]"));
    assert!(report.contains("No correlated issues") || report.contains("100/100"));
}

#[test]
fn test_content_degraded_system_shows_issues() {
    let brain = create_degraded_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 75);

    // Should say degraded
    assert!(report.contains("[SUMMARY]"));
    assert!(report.contains("degraded") || report.contains("stable with warnings"));
    assert!(report.contains("1 critical") && report.contains("1 warning"));

    // Should show health issues
    assert!(report.contains("[HEALTH]"));
    assert!(report.contains("docker.service") || report.contains("failed service"));
}

#[test]
fn test_content_with_proactive_issues() {
    let brain = create_healthy_brain();
    let proactive_issues = vec![
        ProactiveIssueSummaryData {
            summary: "Network priority mismatch detected".to_string(),
            root_cause: "network_priority_mismatch".to_string(),
            severity: "warning".to_string(),
            confidence: 0.85,
            rule_id: Some("network_priority".to_string()),
            first_seen: "2025-11-23T12:00:00Z".to_string(),
            last_seen: "2025-11-23T12:00:00Z".to_string(),
        },
    ];

    let report = compose_sysadmin_report_answer(&brain, None, &proactive_issues, 90);

    // Should show proactive issues
    assert!(report.contains("[PROACTIVE]"));
    assert!(report.contains("Network priority mismatch") || report.contains("90/100"));
    assert!(report.contains("85%") || report.contains("0.85")); // confidence
}

#[test]
fn test_content_with_daily_snapshot() {
    let brain = create_healthy_brain();
    let snapshot = "Kernel: 6.17.8-arch1-1 (unchanged since last session)\nPackages: 1234 (2 new since last session)";

    let report = compose_sysadmin_report_answer(&brain, Some(snapshot), &[], 100);

    // Should include session info
    assert!(report.contains("[SESSION]"));
    assert!(report.contains("Kernel") || report.contains("Packages"));
}

#[test]
fn test_content_key_domains_services() {
    let mut brain = create_healthy_brain();
    brain.insights.push(DiagnosticInsightData {
        rule_id: "check_systemd".to_string(),
        summary: "2 failed services detected".to_string(),
        details: "docker.service, nginx.service".to_string(),
        evidence: "systemctl --failed".to_string(),
        severity: "critical".to_string(),
        commands: vec![],
        citations: vec![],
    });
    brain.critical_count = 1;

    let report = compose_sysadmin_report_answer(&brain, None, &[], 80);

    // Should highlight services in KEY DOMAINS
    assert!(report.contains("[KEY DOMAINS]"));
    assert!(report.contains("Services:") || report.contains("failed service"));
}

#[test]
fn test_content_key_domains_disk() {
    let mut brain = create_healthy_brain();
    brain.insights.push(DiagnosticInsightData {
        rule_id: "check_disk".to_string(),
        summary: "Root partition at 92% capacity".to_string(),
        details: "Critical disk pressure".to_string(),
        evidence: "df -h".to_string(),
        severity: "critical".to_string(),
        commands: vec![],
        citations: vec![],
    });
    brain.critical_count = 1;

    let report = compose_sysadmin_report_answer(&brain, None, &[], 70);

    // Should highlight disk in KEY DOMAINS
    assert!(report.contains("[KEY DOMAINS]"));
    assert!(report.contains("Disk:") || report.contains("partition"));
}

#[test]
fn test_content_key_domains_network() {
    let mut brain = create_healthy_brain();
    brain.insights.push(DiagnosticInsightData {
        rule_id: "check_network".to_string(),
        summary: "Network routing conflict detected".to_string(),
        details: "Multiple default routes".to_string(),
        evidence: "ip route".to_string(),
        severity: "warning".to_string(),
        commands: vec![],
        citations: vec![],
    });
    brain.warning_count = 1;

    let report = compose_sysadmin_report_answer(&brain, None, &[], 85);

    // Should highlight network in KEY DOMAINS
    assert!(report.contains("[KEY DOMAINS]"));
    assert!(report.contains("Network:") || report.contains("routing"));
}

#[test]
fn test_content_commands_critical_system() {
    let brain = create_degraded_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 70);

    // Should suggest remediation commands for critical issues
    assert!(report.contains("[COMMANDS]"));
    assert!(report.contains("check my system health") || report.contains("what should I fix first"));
}

#[test]
fn test_content_commands_healthy_system() {
    let brain = create_healthy_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 100);

    // Should suggest general commands for healthy system
    assert!(report.contains("[COMMANDS]"));
    assert!(report.contains("status") || report.contains("check my"));
}

#[test]
fn test_content_limits_insights_to_three() {
    let mut brain = create_healthy_brain();
    for i in 0..5 {
        brain.insights.push(DiagnosticInsightData {
            rule_id: format!("rule_{}", i),
            summary: format!("Issue {}", i + 1),
            details: "Details".to_string(),
            evidence: "Evidence".to_string(),
            severity: "warning".to_string(),
            commands: vec![],
            citations: vec![],
        });
    }
    brain.warning_count = 5;

    let report = compose_sysadmin_report_answer(&brain, None, &[], 80);

    // Should show max 3 insights + "and N more"
    assert!(report.contains("Issue 1"));
    assert!(report.contains("Issue 2"));
    assert!(report.contains("Issue 3"));
    assert!(report.contains("2 more") || report.contains("and 2 more"));
}

#[test]
fn test_content_limits_proactive_to_three() {
    let brain = create_healthy_brain();
    let proactive_issues = vec![
        ProactiveIssueSummaryData {
            summary: "Issue 1".to_string(),
            root_cause: "root1".to_string(),
            severity: "critical".to_string(),
            confidence: 0.9,
            rule_id: Some("rule1".to_string()),
            first_seen: "2025-11-23T12:00:00Z".to_string(),
            last_seen: "2025-11-23T12:00:00Z".to_string(),
        },
        ProactiveIssueSummaryData {
            summary: "Issue 2".to_string(),
            root_cause: "root2".to_string(),
            severity: "warning".to_string(),
            confidence: 0.8,
            rule_id: Some("rule2".to_string()),
            first_seen: "2025-11-23T12:00:00Z".to_string(),
            last_seen: "2025-11-23T12:00:00Z".to_string(),
        },
        ProactiveIssueSummaryData {
            summary: "Issue 3".to_string(),
            root_cause: "root3".to_string(),
            severity: "warning".to_string(),
            confidence: 0.7,
            rule_id: Some("rule3".to_string()),
            first_seen: "2025-11-23T12:00:00Z".to_string(),
            last_seen: "2025-11-23T12:00:00Z".to_string(),
        },
        ProactiveIssueSummaryData {
            summary: "Issue 4".to_string(),
            root_cause: "root4".to_string(),
            severity: "info".to_string(),
            confidence: 0.6,
            rule_id: Some("rule4".to_string()),
            first_seen: "2025-11-23T12:00:00Z".to_string(),
            last_seen: "2025-11-23T12:00:00Z".to_string(),
        },
    ];

    let report = compose_sysadmin_report_answer(&brain, None, &proactive_issues, 80);

    // Should show max 3 proactive issues + "and N more"
    assert!(report.contains("Issue 1"));
    assert!(report.contains("Issue 2"));
    assert!(report.contains("Issue 3"));
    assert!(report.contains("1 more") || report.contains("and 1 more"));
}

#[test]
fn test_content_priority_critical_over_warning() {
    let mut brain = create_healthy_brain();
    // Add warning first, then critical
    brain.insights.push(DiagnosticInsightData {
        rule_id: "rule1".to_string(),
        summary: "Warning issue".to_string(),
        details: "Details".to_string(),
        evidence: "Evidence".to_string(),
        severity: "warning".to_string(),
        commands: vec![],
        citations: vec![],
    });
    brain.insights.push(DiagnosticInsightData {
        rule_id: "rule2".to_string(),
        summary: "Critical issue".to_string(),
        details: "Details".to_string(),
        evidence: "Evidence".to_string(),
        severity: "critical".to_string(),
        commands: vec![],
        citations: vec![],
    });
    brain.critical_count = 1;
    brain.warning_count = 1;

    let report = compose_sysadmin_report_answer(&brain, None, &[], 75);

    // Critical issue should appear before warning in output
    let critical_pos = report.find("Critical issue").expect("Should contain critical issue");
    let warning_pos = report.find("Warning issue").expect("Should contain warning issue");
    assert!(critical_pos < warning_pos, "Critical should come before warning");
}

// ===========================================================================
// SECTION 3: FORMATTING TESTS (6 tests)
// ===========================================================================

#[test]
fn test_format_all_sections_present_healthy() {
    let brain = create_healthy_brain();
    let snapshot = "Kernel: 6.17.8-arch1-1";
    let report = compose_sysadmin_report_answer(&brain, Some(snapshot), &[], 100);

    // All 6 sections should be present
    assert!(report.contains("[SUMMARY]"));
    assert!(report.contains("[HEALTH]"));
    assert!(report.contains("[SESSION]"));
    assert!(report.contains("[PROACTIVE]"));
    assert!(report.contains("[KEY DOMAINS]"));
    assert!(report.contains("[COMMANDS]"));
}

#[test]
fn test_format_all_sections_present_degraded() {
    let brain = create_degraded_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 75);

    // All sections should be present (SESSION may be empty but [SUMMARY], [HEALTH], etc should exist)
    assert!(report.contains("[SUMMARY]"));
    assert!(report.contains("[HEALTH]"));
    assert!(report.contains("[PROACTIVE]"));
    assert!(report.contains("[KEY DOMAINS]"));
    assert!(report.contains("[COMMANDS]"));
}

#[test]
fn test_format_severity_markers() {
    let brain = create_degraded_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 75);

    // Should use canonical severity markers
    // ✗ for critical, ⚠ for warning, ℹ for info
    assert!(report.contains("✗") || report.contains("⚠")); // at least one marker
}

#[test]
fn test_format_no_internal_types_leaked() {
    let brain = create_degraded_brain();
    let report = compose_sysadmin_report_answer(&brain, None, &[], 75);

    // Should not leak internal Rust types or debug formatting
    assert!(!report.contains("BrainAnalysisData"));
    assert!(!report.contains("DiagnosticInsightData"));
    assert!(!report.contains("Vec<"));
    assert!(!report.contains("Option<"));
    assert!(!report.contains("{:?}"));
}

#[test]
fn test_format_concise_report_length() {
    let brain = create_degraded_brain();
    let snapshot = "Kernel: 6.17.8-arch1-1\nPackages: 1234 (2 new)";
    let proactive = vec![
        ProactiveIssueSummaryData {
            summary: "Network issue".to_string(),
            root_cause: "network_priority".to_string(),
            severity: "warning".to_string(),
            confidence: 0.8,
            rule_id: Some("network_rule".to_string()),
            first_seen: "2025-11-23T12:00:00Z".to_string(),
            last_seen: "2025-11-23T12:00:00Z".to_string(),
        },
    ];

    let report = compose_sysadmin_report_answer(&brain, Some(snapshot), &proactive, 75);

    // Target: 20-40 lines max (should be concise, not a novel)
    let line_count = report.lines().count();
    assert!(line_count <= 50, "Report should be concise (<= 50 lines), got {} lines", line_count);
}

#[test]
fn test_format_section_order() {
    let brain = create_degraded_brain();
    let snapshot = "Kernel: 6.17.8-arch1-1";
    let report = compose_sysadmin_report_answer(&brain, Some(snapshot), &[], 75);

    // Sections should appear in correct order
    let summary_pos = report.find("[SUMMARY]").expect("Should have SUMMARY");
    let health_pos = report.find("[HEALTH]").expect("Should have HEALTH");
    let proactive_pos = report.find("[PROACTIVE]").expect("Should have PROACTIVE");
    let domains_pos = report.find("[KEY DOMAINS]").expect("Should have KEY DOMAINS");
    let commands_pos = report.find("[COMMANDS]").expect("Should have COMMANDS");

    assert!(summary_pos < health_pos, "SUMMARY before HEALTH");
    assert!(health_pos < proactive_pos, "HEALTH before PROACTIVE");
    assert!(proactive_pos < domains_pos, "PROACTIVE before KEY DOMAINS");
    assert!(domains_pos < commands_pos, "KEY DOMAINS before COMMANDS");
}

// ===========================================================================
// Summary: 26 tests total
// - 8 routing tests
// - 12 content tests
// - 6 formatting tests
// ===========================================================================
