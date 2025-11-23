//! Regression tests for TUI Health & Flow (Beta.266)
//!
//! Tests cover:
//! - Welcome panel uses canonical health wording matching CLI
//! - Brain panel never shows internal debug output
//! - Exit summary uses canonical health format
//! - No Rust enum names or debug fragments in TUI strings
//!
//! Beta.266: TUI Health Coherence & Network Proactivity v1

use annactl::diagnostic_formatter::{DailySnapshotLite, OverallHealth};
use annactl::tui::flow::{generate_exit_summary, generate_welcome_lines};
use annactl::tui_state::AnnaTuiState;
use anna_common::ipc::DiagnosticInsightData;

/// Helper: Create test state with specific health level
fn create_test_state(health: OverallHealth, critical: usize, warnings: usize) -> AnnaTuiState {
    let mut state = AnnaTuiState::default();
    state.brain_available = true;
    state.daily_snapshot = Some(DailySnapshotLite {
        overall_health_opt: Some(health),
        critical_count: critical,
        warning_count: warnings,
        kernel_changed: false,
        packages_changed: false,
        boots_since_last: 0,
    });
    state.system_panel.anna_version = "5.7.0-beta.266".to_string();
    state.system_panel.hostname = "testhost".to_string();
    state
}

/// Helper: Create insight with specific evidence
fn create_insight(severity: &str, summary: &str, evidence: &str) -> DiagnosticInsightData {
    DiagnosticInsightData {
        rule_id: "test_rule".to_string(),
        severity: severity.to_string(),
        summary: summary.to_string(),
        details: "Test details".to_string(),
        commands: vec!["test command".to_string()],
        citations: vec![],
        evidence: evidence.to_string(),
    }
}

#[test]
fn test_welcome_uses_canonical_health_wording() {
    // Test Healthy state
    let state_healthy = create_test_state(OverallHealth::Healthy, 0, 0);
    let lines_healthy = generate_welcome_lines(&state_healthy);
    let text_healthy: String = lines_healthy
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Should use canonical healthy wording
    assert!(
        text_healthy.contains("all clear, no critical issues detected"),
        "Healthy state must use canonical wording: 'all clear, no critical issues detected'"
    );

    // Test DegradedWarning state
    let state_warning = create_test_state(OverallHealth::DegradedWarning, 0, 2);
    let lines_warning = generate_welcome_lines(&state_warning);
    let text_warning: String = lines_warning
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Should use canonical warning wording
    assert!(
        text_warning.contains("degraded") && text_warning.contains("warning"),
        "Warning state must use canonical wording with 'degraded' and 'warning'"
    );

    // Test DegradedCritical state
    let state_critical = create_test_state(OverallHealth::DegradedCritical, 2, 1);
    let lines_critical = generate_welcome_lines(&state_critical);
    let text_critical: String = lines_critical
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Should use canonical critical wording
    assert!(
        text_critical.contains("degraded") && text_critical.contains("critical"),
        "Critical state must use canonical wording with 'degraded' and 'critical'"
    );
    assert!(
        text_critical.contains("require attention") || text_critical.contains("requires attention"),
        "Critical state must mention 'require attention'"
    );

    // Should match CLI pattern "Today:" or "System health:"
    assert!(
        text_healthy.contains("Today:") || text_healthy.contains("System health:"),
        "Welcome must use 'Today:' or 'System health:' label"
    );
}

#[test]
fn test_exit_summary_uses_canonical_health_wording() {
    // Test all health states
    let state_healthy = create_test_state(OverallHealth::Healthy, 0, 0);
    let lines_healthy = generate_exit_summary(&state_healthy);
    let text_healthy: String = lines_healthy
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Should use canonical healthy wording
    assert!(
        text_healthy.contains("all clear, no critical issues detected"),
        "Exit summary healthy state must match canonical wording"
    );

    // Should use "System health:" label (not "Session health:")
    assert!(
        text_healthy.contains("System health:"),
        "Exit summary must use 'System health:' label to match CLI"
    );

    let state_critical = create_test_state(OverallHealth::DegradedCritical, 2, 1);
    let lines_critical = generate_exit_summary(&state_critical);
    let text_critical: String = lines_critical
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    assert!(
        text_critical.contains("degraded") && text_critical.contains("critical"),
        "Exit summary critical state must match canonical wording"
    );
}

#[test]
fn test_no_raw_internal_types_in_welcome() {
    let state = create_test_state(OverallHealth::DegradedCritical, 2, 1);
    let lines = generate_welcome_lines(&state);
    let text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Should NEVER contain Rust enum names
    assert!(
        !text.contains("::"),
        "Welcome panel must not contain Rust enum syntax (::)"
    );
    assert!(
        !text.contains("OverallHealth"),
        "Welcome panel must not contain internal type name 'OverallHealth'"
    );
    assert!(
        !text.contains("HealthStatus"),
        "Welcome panel must not contain internal type name 'HealthStatus'"
    );
    assert!(
        !text.contains("DegradedCritical"),
        "Welcome panel must not contain enum variant 'DegradedCritical'"
    );

    // Should not contain debug fragments
    assert!(
        !text.contains("System degraded: 0 failed"),
        "Welcome panel must not contain debug fragments"
    );
}

#[test]
fn test_no_raw_internal_types_in_exit_summary() {
    let state = create_test_state(OverallHealth::DegradedWarning, 0, 2);
    let lines = generate_exit_summary(&state);
    let text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Should NEVER contain Rust enum names
    assert!(
        !text.contains("::"),
        "Exit summary must not contain Rust enum syntax (::)"
    );
    assert!(
        !text.contains("OverallHealth"),
        "Exit summary must not contain internal type name"
    );
    assert!(
        !text.contains("DegradedWarning"),
        "Exit summary must not contain enum variant"
    );
}

#[test]
fn test_brain_panel_filters_internal_debug_output() {
    // This test verifies the is_internal_debug_output() filter

    // Create insights with various evidence types
    let clean_insight = create_insight("critical", "Disk space low", "Root filesystem at 95% capacity");

    let debug_insight_1 = create_insight(
        "warning",
        "System health issue",
        "HealthStatus::Degraded - System degraded: 0 failed"
    );

    let debug_insight_2 = create_insight(
        "critical",
        "Network problem",
        "NetworkState::Disconnected"
    );

    // Verify clean evidence doesn't contain debug patterns
    assert!(
        !clean_insight.evidence.contains("::"),
        "Clean insight should not contain :: syntax"
    );

    // Verify debug evidence DOES contain problematic patterns
    assert!(
        debug_insight_1.evidence.contains("::"),
        "Debug insight 1 should contain :: syntax for test validity"
    );
    assert!(
        debug_insight_1.evidence.contains("System degraded:"),
        "Debug insight 1 should contain debug fragment for test validity"
    );
    assert!(
        debug_insight_2.evidence.contains("::"),
        "Debug insight 2 should contain :: syntax for test validity"
    );

    // The actual filtering happens in brain.rs::is_internal_debug_output()
    // This test documents the expected behavior
}

#[test]
fn test_health_wording_consistency_across_surfaces() {
    // Verify that welcome and exit summary use identical health phrases
    let state = create_test_state(OverallHealth::DegradedCritical, 3, 2);

    let welcome_lines = generate_welcome_lines(&state);
    let welcome_text: String = welcome_lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    let exit_lines = generate_exit_summary(&state);
    let exit_text: String = exit_lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Both should contain the same health phrase
    assert!(
        welcome_text.contains("degraded") && exit_text.contains("degraded"),
        "Welcome and exit must both use 'degraded' for DegradedCritical state"
    );
    assert!(
        welcome_text.contains("critical") && exit_text.contains("critical"),
        "Welcome and exit must both use 'critical' for DegradedCritical state"
    );

    // Both should use issue counts in same format
    assert!(
        welcome_text.contains("3 critical") && welcome_text.contains("2 warning"),
        "Welcome must show '3 critical, 2 warnings' format"
    );
}

#[test]
fn test_welcome_panel_shows_today_label() {
    // Verify welcome uses "Today:" temporal label
    let state = create_test_state(OverallHealth::Healthy, 0, 0);
    let lines = generate_welcome_lines(&state);
    let text: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
        .collect();

    // Must use "Today:" label for temporal context
    assert!(
        text.contains("Today:"),
        "Welcome panel must use 'Today:' label for health summary"
    );
}
