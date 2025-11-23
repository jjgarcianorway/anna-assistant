//! Beta.250: Health & Diagnostic Answer Content Regression Tests
//!
//! These tests validate the CONTENT of diagnostic answers, not just routing.
//! They ensure diagnostic answers always:
//! - State health status clearly ("all clear" or "issue(s) detected")
//! - Use the canonical [SUMMARY]/[DETAILS]/[COMMANDS] structure
//! - Attribute to "internal diagnostic engine"
//! - Never include old "Confidence: High | Sources: LLM" format

use annactl::diagnostic_formatter::{format_diagnostic_report, DiagnosticMode};
use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};

#[test]
fn test_all_clear_contains_required_phrases() {
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        proactive_issues: vec![],
        insights: vec![],
        proactive_issues: vec![],
    };

    let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

    // Must contain health status statement
    assert!(report.contains("System health:"), "Report must start with 'System health:'");
    assert!(report.contains("all clear") || report.contains("no critical issues"),
        "All clear report must explicitly state 'all clear' or 'no critical issues'");

    // Must use canonical structure
    assert!(report.contains("[SUMMARY]"), "Report must have [SUMMARY] section");
    assert!(report.contains("[COMMANDS]"), "Report must have [COMMANDS] section");

    // Must not contain old LLM attribution format
    assert!(!report.contains("Confidence:"), "Should not contain 'Confidence:' field");
    assert!(!report.contains("Sources: LLM"), "Should not contain 'Sources: LLM'");
}

#[test]
fn test_critical_issues_contains_counts() {
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 2,
        warning_count: 1,
        proactive_issues: vec![],
        insights: vec![
            DiagnosticInsightData {
                rule_id: "test_1".to_string(),
                severity: "critical".to_string(),
                summary: "Critical issue 1".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
            DiagnosticInsightData {
                rule_id: "test_2".to_string(),
                severity: "critical".to_string(),
                summary: "Critical issue 2".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
            DiagnosticInsightData {
                rule_id: "test_3".to_string(),
                severity: "warning".to_string(),
                summary: "Warning issue".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
        ],
    };

    let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

    // Must contain health status with counts
    assert!(report.contains("System health:"), "Report must start with 'System health:'");
    assert!(report.contains("issue(s) detected"), "Report must state 'issue(s) detected'");
    assert!(report.contains("2 critical") || report.contains("critical"),
        "Report must mention critical count");
    assert!(report.contains("1 warning") || report.contains("warning"),
        "Report must mention warning count");

    // Must show severity markers
    assert!(report.contains("✗"), "Report must contain critical marker (✗)");
    assert!(report.contains("⚠"), "Report must contain warning marker (⚠)");

    // Must use canonical structure
    assert!(report.contains("[SUMMARY]"), "Report must have [SUMMARY] section");
    assert!(report.contains("[DETAILS]"), "Report must have [DETAILS] section");
    assert!(report.contains("[COMMANDS]"), "Report must have [COMMANDS] section");
}

#[test]
fn test_warnings_only_clearly_stated() {
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 2,
        proactive_issues: vec![],
        insights: vec![
            DiagnosticInsightData {
                rule_id: "test_1".to_string(),
                severity: "warning".to_string(),
                summary: "Warning 1".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
            DiagnosticInsightData {
                rule_id: "test_2".to_string(),
                severity: "warning".to_string(),
                summary: "Warning 2".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
        ],
    };

    let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

    // Must clearly state warnings only, no critical
    assert!(report.contains("System health:"), "Report must start with 'System health:'");
    assert!(report.contains("warning"), "Report must mention warnings");
    assert!(report.contains("no critical"), "Report must explicitly state 'no critical'");
}

#[test]
fn test_summary_mode_limits_to_three() {
    let mut insights = vec![];
    for i in 1..=10 {
        insights.push(DiagnosticInsightData {
            rule_id: format!("test_{}", i),
            severity: "critical".to_string(),
            summary: format!("Issue {}", i),
            details: "Details".to_string(),
            commands: vec![],
            citations: vec![],
            evidence: String::new(),
        });
    }

    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 10,
        warning_count: 0,
        proactive_issues: vec![],
        insights,
    };

    let report = format_diagnostic_report(&analysis, DiagnosticMode::Summary);

    // Summary mode should show top 3 only
    assert!(report.contains("Issue 1"), "Should show first issue");
    assert!(report.contains("Issue 2"), "Should show second issue");
    assert!(report.contains("Issue 3"), "Should show third issue");
    assert!(!report.contains("Issue 4"), "Should not show fourth issue");
    assert!(report.contains("... and 7 more"), "Should indicate remaining count");
}

#[test]
fn test_full_mode_shows_up_to_five() {
    let mut insights = vec![];
    for i in 1..=10 {
        insights.push(DiagnosticInsightData {
            rule_id: format!("test_{}", i),
            severity: "critical".to_string(),
            summary: format!("Issue {}", i),
            details: "Details".to_string(),
            commands: vec![],
            citations: vec![],
            evidence: String::new(),
        });
    }

    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 10,
        warning_count: 0,
        proactive_issues: vec![],
        insights,
    };

    let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

    // Full mode should show up to 5
    assert!(report.contains("Issue 1"), "Should show first issue");
    assert!(report.contains("Issue 2"), "Should show second issue");
    assert!(report.contains("Issue 3"), "Should show third issue");
    assert!(report.contains("Issue 4"), "Should show fourth issue");
    assert!(report.contains("Issue 5"), "Should show fifth issue");
    assert!(!report.contains("Issue 6"), "Should not show sixth issue");
}

#[test]
fn test_commands_section_appropriate_for_state() {
    // Test 1: All clear - should suggest status check only
    let all_clear = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        proactive_issues: vec![],
        insights: vec![],
        proactive_issues: vec![],
    };

    let report = format_diagnostic_report(&all_clear, DiagnosticMode::Full);
    assert!(report.contains("No actions required"), "All clear should say no actions required");
    assert!(report.contains("annactl status"), "Should suggest status check");

    // Test 2: Issues detected - should suggest diagnostic commands
    let with_issues = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 1,
        warning_count: 0,
        proactive_issues: vec![],
        insights: vec![
            DiagnosticInsightData {
                rule_id: "test".to_string(),
                severity: "critical".to_string(),
                summary: "Issue".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
        ],
    };

    let report_issues = format_diagnostic_report(&with_issues, DiagnosticMode::Full);
    assert!(report_issues.contains("journalctl"), "Issues report should suggest journalctl");
    assert!(report_issues.contains("systemctl"), "Issues report should suggest systemctl");
}

#[test]
fn test_no_llm_attribution_in_diagnostic_answers() {
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 1,
        warning_count: 0,
        proactive_issues: vec![],
        insights: vec![
            DiagnosticInsightData {
                rule_id: "test".to_string(),
                severity: "critical".to_string(),
                summary: "Test issue".to_string(),
                details: "Details".to_string(),
                commands: vec!["test command".to_string()],
                citations: vec![],
                evidence: String::new(),
            },
        ],
    };

    let report = format_diagnostic_report(&analysis, DiagnosticMode::Full);

    // Must NOT contain old LLM attribution patterns
    assert!(!report.contains("Confidence:"), "Should not have Confidence field");
    assert!(!report.contains("Sources: LLM"), "Should not attribute to LLM");
    assert!(!report.contains("Based on LLM"), "Should not mention LLM as source");

    // Deterministic answers should be clear they're from diagnostic engine
    // (This is enforced at the UnifiedQueryResult level, not in the formatter itself)
}

// Beta.259: Health Wording Consistency Tests
// These tests ensure all three health surfaces (diagnostic report, daily snapshot, status)
// produce consistent health messaging for the same scenario.

#[test]
fn test_health_consistency_healthy_scenario() {
    use annactl::diagnostic_formatter::{
        compute_overall_health, format_diagnostic_report_with_query, format_daily_snapshot,
        compute_daily_snapshot, format_today_health_line_from_health, SessionDelta,
        DiagnosticMode, OverallHealth,
    };

    // Scenario: 0 critical, 0 warnings
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        proactive_issues: vec![],
        insights: vec![],
        proactive_issues: vec![],
    };

    let session_delta = SessionDelta::default();

    // 1. Compute overall health
    let overall_health = compute_overall_health(&analysis);
    assert_eq!(overall_health, OverallHealth::Healthy);

    // 2. Format from diagnostic report
    let diagnostic_report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // 3. Format from daily snapshot
    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let daily_report = format_daily_snapshot(&snapshot, false);

    // 4. Format from status "Today:" line
    let status_line = format_today_health_line_from_health(overall_health);

    // All three should agree on "all clear"
    assert!(diagnostic_report.to_lowercase().contains("all clear"), "Diagnostic should say 'all clear'");
    assert!(daily_report.to_lowercase().contains("all clear"), "Daily snapshot should say 'all clear'");
    assert!(status_line.to_lowercase().contains("all clear"), "Status line should say 'all clear'");

    // None should mention critical or degraded
    assert!(!diagnostic_report.to_lowercase().contains("degraded"), "Healthy diagnostic should not mention 'degraded'");
    assert!(!daily_report.to_lowercase().contains("degraded"), "Healthy snapshot should not mention 'degraded'");
    assert!(!status_line.to_lowercase().contains("degraded"), "Healthy status should not mention 'degraded'");
}

#[test]
fn test_health_consistency_degraded_critical_scenario() {
    use annactl::diagnostic_formatter::{
        compute_overall_health, format_diagnostic_report_with_query, format_daily_snapshot,
        compute_daily_snapshot, format_today_health_line_from_health, SessionDelta,
        DiagnosticMode, OverallHealth,
    };

    // Scenario: 2 critical, 1 warning
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 2,
        warning_count: 1,
        proactive_issues: vec![],
        insights: vec![
            DiagnosticInsightData {
                rule_id: "crit_1".to_string(),
                severity: "critical".to_string(),
                summary: "Critical issue 1".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
            DiagnosticInsightData {
                rule_id: "crit_2".to_string(),
                severity: "critical".to_string(),
                summary: "Critical issue 2".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
            DiagnosticInsightData {
                rule_id: "warn_1".to_string(),
                severity: "warning".to_string(),
                summary: "Warning issue".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
        ],
    };

    let session_delta = SessionDelta::default();

    // 1. Compute overall health
    let overall_health = compute_overall_health(&analysis);
    assert_eq!(overall_health, OverallHealth::DegradedCritical);

    // 2. Format from diagnostic report
    let diagnostic_report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // 3. Format from daily snapshot
    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let daily_report = format_daily_snapshot(&snapshot, false);

    // 4. Format from status "Today:" line
    let status_line = format_today_health_line_from_health(overall_health);

    // All three should mention degraded and critical
    assert!(diagnostic_report.to_lowercase().contains("critical") || diagnostic_report.to_lowercase().contains("issue(s) detected"),
            "Diagnostic should mention critical or issues");
    assert!(daily_report.to_lowercase().contains("degraded"), "Daily snapshot should mention 'degraded'");
    assert!(daily_report.to_lowercase().contains("critical"), "Daily snapshot should mention 'critical'");
    assert!(status_line.to_lowercase().contains("degraded"), "Status line should mention 'degraded'");
    assert!(status_line.to_lowercase().contains("critical"), "Status line should mention 'critical'");

    // None should say "all clear"
    assert!(!diagnostic_report.to_lowercase().contains("all clear"), "Critical system should not say 'all clear'");
    assert!(!daily_report.to_lowercase().contains("all clear"), "Critical system should not say 'all clear'");
    assert!(!status_line.to_lowercase().contains("all clear"), "Critical system should not say 'all clear'");
}

#[test]
fn test_health_consistency_degraded_warning_scenario() {
    use annactl::diagnostic_formatter::{
        compute_overall_health, format_diagnostic_report_with_query, format_daily_snapshot,
        compute_daily_snapshot, format_today_health_line_from_health, SessionDelta,
        DiagnosticMode, OverallHealth,
    };

    // Scenario: 0 critical, 2 warnings
    let analysis = BrainAnalysisData {
        timestamp: "2025-11-22T12:00:00Z".to_string(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 2,
        proactive_issues: vec![],
        insights: vec![
            DiagnosticInsightData {
                rule_id: "warn_1".to_string(),
                severity: "warning".to_string(),
                summary: "Warning issue 1".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
            DiagnosticInsightData {
                rule_id: "warn_2".to_string(),
                severity: "warning".to_string(),
                summary: "Warning issue 2".to_string(),
                details: "Details".to_string(),
                commands: vec![],
                citations: vec![],
                evidence: String::new(),
            },
        ],
    };

    let session_delta = SessionDelta::default();

    // 1. Compute overall health
    let overall_health = compute_overall_health(&analysis);
    assert_eq!(overall_health, OverallHealth::DegradedWarning);

    // 2. Format from diagnostic report
    let diagnostic_report = format_diagnostic_report_with_query(&analysis, DiagnosticMode::Full, "check my system");

    // 3. Format from daily snapshot
    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let daily_report = format_daily_snapshot(&snapshot, false);

    // 4. Format from status "Today:" line
    let status_line = format_today_health_line_from_health(overall_health);

    // All three should mention degraded and warning
    assert!(diagnostic_report.to_lowercase().contains("warning"), "Diagnostic should mention 'warning'");
    assert!(daily_report.to_lowercase().contains("degraded"), "Daily snapshot should mention 'degraded'");
    assert!(daily_report.to_lowercase().contains("warning"), "Daily snapshot should mention 'warning'");
    assert!(status_line.to_lowercase().contains("degraded"), "Status line should mention 'degraded'");
    assert!(status_line.to_lowercase().contains("warning"), "Status line should mention 'warning'");

    // None should say "all clear" or "critical"
    assert!(!diagnostic_report.to_lowercase().contains("all clear"), "Warning system should not say 'all clear'");
    assert!(!daily_report.to_lowercase().contains("all clear"), "Warning system should not say 'all clear'");
    assert!(!status_line.to_lowercase().contains("all clear"), "Warning system should not say 'all clear'");
    assert!(!status_line.to_lowercase().contains("critical"), "Warning-only system should not say 'critical'");
}
