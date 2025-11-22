//! Beta.258: Daily Snapshot Content Regression Tests
//!
//! These tests validate the content and structure of daily snapshot answers.
//! They ensure:
//! - Temporal wording for "today" queries
//! - Session delta information (kernel, packages, boots)
//! - Issue summaries in compact format
//! - Graceful degradation when session metadata unavailable

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use annactl::diagnostic_formatter::{
    compute_daily_snapshot, format_daily_snapshot, OverallHealth, SessionDelta,
};

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
fn test_healthy_today_snapshot() {
    let analysis = mock_analysis(0, 0);
    let session_delta = SessionDelta {
        kernel_changed: false,
        old_kernel: None,
        new_kernel: None,
        package_delta: 0,
        boots_since_last: 0,
    };

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    assert_eq!(snapshot.overall_health, OverallHealth::Healthy);

    let report = format_daily_snapshot(&snapshot, true);

    // Should use temporal wording
    assert!(report.contains("System health today:"), "Should use 'today' wording");
    assert!(report.contains("all clear"), "Should mention 'all clear'");
    assert!(report.contains("no critical issues detected"), "Should state no critical issues");

    // Should contain session delta section
    assert!(report.contains("[SESSION DELTA]"), "Should have session delta section");
    assert!(report.contains("Kernel: unchanged since last session"), "Should mention unchanged kernel");
    assert!(report.contains("Packages: no changes since last session"), "Should mention no package changes");
    assert!(report.contains("Boots: no reboots since last session"), "Should mention no reboots");
    assert!(report.contains("Issues: 0 critical, 0 warnings"), "Should show zero issues");
}

#[test]
fn test_degraded_critical_today_snapshot() {
    let analysis = mock_analysis(2, 1);
    let session_delta = SessionDelta {
        kernel_changed: false,
        old_kernel: None,
        new_kernel: None,
        package_delta: 0,
        boots_since_last: 0,
    };

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    assert_eq!(snapshot.overall_health, OverallHealth::DegradedCritical);

    let report = format_daily_snapshot(&snapshot, true);

    // Should mention degraded and critical
    assert!(report.contains("System health today:"), "Should use 'today' wording");
    assert!(report.contains("degraded"), "Should mention 'degraded'");
    assert!(report.contains("critical issues require attention"), "Should mention critical attention needed");

    // Should show issue counts
    assert!(report.contains("Issues: 2 critical, 1 warning(s)"), "Should show correct counts");

    // Should show top issues
    assert!(report.contains("[TOP ISSUES]"), "Should have top issues section");
    assert!(report.contains("✗"), "Should contain critical marker");
}

#[test]
fn test_degraded_warning_today_snapshot() {
    let analysis = mock_analysis(0, 2);
    let session_delta = SessionDelta {
        kernel_changed: false,
        old_kernel: None,
        new_kernel: None,
        package_delta: 0,
        boots_since_last: 0,
    };

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    assert_eq!(snapshot.overall_health, OverallHealth::DegradedWarning);

    let report = format_daily_snapshot(&snapshot, true);

    // Should mention degraded and warning
    assert!(report.contains("degraded"), "Should mention 'degraded'");
    assert!(report.contains("warning issues detected"), "Should mention warning issues");
    assert!(report.contains("Issues: 0 critical, 2 warning(s)"), "Should show correct counts");
}

#[test]
fn test_kernel_changed_snapshot() {
    let analysis = mock_analysis(0, 0);
    let session_delta = SessionDelta {
        kernel_changed: true,
        old_kernel: Some("6.17.7".to_string()),
        new_kernel: Some("6.17.8".to_string()),
        package_delta: 0,
        boots_since_last: 1,
    };

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let report = format_daily_snapshot(&snapshot, true);

    // Should show kernel update
    assert!(report.contains("Kernel: updated since last session"), "Should mention kernel update");
    assert!(report.contains("6.17.7 → 6.17.8"), "Should show kernel version transition");

    // Should show 1 reboot
    assert!(report.contains("Boots: 1 reboot since last session"), "Should mention 1 reboot");
}

#[test]
fn test_package_upgrades_snapshot() {
    let analysis = mock_analysis(0, 0);
    let session_delta = SessionDelta {
        kernel_changed: false,
        old_kernel: None,
        new_kernel: None,
        package_delta: 15,
        boots_since_last: 0,
    };

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let report = format_daily_snapshot(&snapshot, true);

    // Should show package upgrades
    assert!(report.contains("Packages: 15 package(s) upgraded"), "Should show package count");
}

#[test]
fn test_non_temporal_snapshot() {
    let analysis = mock_analysis(0, 0);
    let session_delta = SessionDelta::default();

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let report = format_daily_snapshot(&snapshot, false);

    // Should NOT use temporal wording
    assert!(report.contains("System health:"), "Should use generic 'System health:'");
    assert!(!report.contains("System health today:"), "Should NOT use 'today' wording");
}

#[test]
fn test_no_session_metadata_graceful_degradation() {
    let analysis = mock_analysis(1, 0);
    let session_delta = SessionDelta::default(); // All defaults = first run or missing metadata

    let snapshot = compute_daily_snapshot(&analysis, session_delta);
    let report = format_daily_snapshot(&snapshot, true);

    // Should still work with defaults
    assert!(report.contains("System health today:"), "Should use temporal wording");
    assert!(report.contains("degraded"), "Should show health status");
    assert!(report.contains("Kernel: unchanged since last session"), "Should show default kernel status");
    assert!(report.contains("Packages: no changes since last session"), "Should show default package status");
    assert!(report.contains("Boots: no reboots since last session"), "Should show default boot status");
}

#[test]
fn test_snapshot_includes_top_issues() {
    let analysis = mock_analysis(2, 1);
    let session_delta = SessionDelta::default();

    let snapshot = compute_daily_snapshot(&analysis, session_delta);

    // Should extract top 3 issue summaries
    assert_eq!(snapshot.top_issue_summaries.len(), 3, "Should have 3 issue summaries");
    assert_eq!(snapshot.top_issue_summaries[0], "Critical issue 0");
    assert_eq!(snapshot.top_issue_summaries[1], "Critical issue 1");
    assert_eq!(snapshot.top_issue_summaries[2], "Warning issue 0");

    let report = format_daily_snapshot(&snapshot, true);

    // Should display them in report
    assert!(report.contains("Critical issue 0"), "Should show first critical");
    assert!(report.contains("Critical issue 1"), "Should show second critical");
    assert!(report.contains("Warning issue 0"), "Should show warning");
}
