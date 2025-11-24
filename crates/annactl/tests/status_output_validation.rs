//! Status Output Validation Tests (6.17.0)
//!
//! Validates that `annactl status` output is consistent and truthful.
//!
//! Key requirements:
//! 1. Overall status must NEVER be "HEALTHY" when there are warnings or critical issues
//! 2. Daemon health shows red if RPC socket is unreachable (even if systemd says active)
//! 3. Permission checks are consistent across sections
//! 4. Brain analysis unavailability appears as a diagnostic
//! 5. /var/log/anna permission issues are reported with fix commands

use annactl::status_health::{
    check_daemon_health, check_brain_analysis, HealthSummary, DiagnosticSeverity, HealthLevel,
};

#[tokio::test]
async fn test_daemon_health_requires_both_systemd_and_rpc() {
    // Case 1: Systemd active but RPC unreachable should be CRITICAL
    let diag = check_daemon_health(true, true, false).await;
    assert!(diag.is_some());
    let diag = diag.unwrap();
    assert_eq!(diag.severity, DiagnosticSeverity::Critical);
    assert!(diag.title.contains("not reachable"));
    assert!(diag.body.contains("/run/anna.sock"));
    assert!(!diag.hints.is_empty());

    // Case 2: Systemd inactive should be CRITICAL
    let diag = check_daemon_health(false, true, false).await;
    assert!(diag.is_some());
    let diag = diag.unwrap();
    assert_eq!(diag.severity, DiagnosticSeverity::Critical);
    assert!(diag.title.contains("not running"));

    // Case 3: Systemd active, RPC reachable, but not enabled should be WARNING
    let diag = check_daemon_health(true, false, true).await;
    assert!(diag.is_some());
    let diag = diag.unwrap();
    assert_eq!(diag.severity, DiagnosticSeverity::Warning);
    assert!(diag.title.contains("not enabled"));

    // Case 4: All OK should return None
    let diag = check_daemon_health(true, true, true).await;
    assert!(diag.is_none());
}

#[test]
fn test_brain_analysis_unavailable_is_critical() {
    let diag = check_brain_analysis(None);
    assert!(diag.is_some());
    let diag = diag.unwrap();
    assert_eq!(diag.severity, DiagnosticSeverity::Critical);
    assert!(diag.title.contains("Brain analysis unavailable"));
    assert!(diag.body.contains("annad"));
    assert!(!diag.hints.is_empty());
}

#[test]
fn test_health_summary_never_healthy_with_warnings() {
    let mut summary = HealthSummary::new();

    // Add a warning
    summary.add_diagnostic(annactl::status_health::Diagnostic {
        title: "Test warning".to_string(),
        body: "Test body".to_string(),
        severity: DiagnosticSeverity::Warning,
        hints: vec![],
    });

    summary.compute_level();

    // Must NOT be healthy
    assert_ne!(summary.level, HealthLevel::Healthy);
    assert_eq!(summary.level, HealthLevel::Degraded);
    assert_eq!(summary.warning_count, 1);
}

#[test]
fn test_health_summary_critical_with_critical_issues() {
    let mut summary = HealthSummary::new();

    // Add a critical issue
    summary.add_diagnostic(annactl::status_health::Diagnostic {
        title: "Test critical".to_string(),
        body: "Test body".to_string(),
        severity: DiagnosticSeverity::Critical,
        hints: vec![],
    });

    summary.compute_level();

    // Must be critical
    assert_eq!(summary.level, HealthLevel::Critical);
    assert_eq!(summary.critical_count, 1);
}

#[test]
fn test_health_summary_healthy_only_when_zero_issues() {
    let mut summary = HealthSummary::new();

    // No diagnostics added
    summary.compute_level();

    // Should be healthy
    assert_eq!(summary.level, HealthLevel::Healthy);
    assert_eq!(summary.critical_count, 0);
    assert_eq!(summary.warning_count, 0);
}

#[test]
fn test_health_summary_status_line() {
    let mut summary = HealthSummary::new();

    // Healthy case
    summary.compute_level();
    assert!(summary.status_line().contains("all systems operational"));

    // Degraded case
    summary.add_diagnostic(annactl::status_health::Diagnostic {
        title: "Test".to_string(),
        body: "Test".to_string(),
        severity: DiagnosticSeverity::Warning,
        hints: vec![],
    });
    summary.compute_level();
    let status = summary.status_line();
    assert!(status.contains("warning"));

    // Critical case
    let mut summary2 = HealthSummary::new();
    summary2.add_diagnostic(annactl::status_health::Diagnostic {
        title: "Test".to_string(),
        body: "Test".to_string(),
        severity: DiagnosticSeverity::Critical,
        hints: vec![],
    });
    summary2.compute_level();
    let status2 = summary2.status_line();
    assert!(status2.contains("critical"));
}
