//! TUI Flow Module (Beta.261)
//!
//! Handles startup welcome, exit summary, and shared diagnostic hints.
//!
//! Philosophy:
//! - Reuse existing welcome engine and health data (no new RPC calls)
//! - Use canonical answer formatting and TuiStyles for consistency
//! - Keep messages compact and deterministic
//! - Align hints between CLI and TUI

use crate::diagnostic_formatter::{compute_overall_health, OverallHealth};
use crate::tui_state::AnnaTuiState;
use anna_common::ipc::BrainAnalysisData;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::formatting::TuiStyles;

/// Beta.261: Generate startup welcome lines for TUI conversation
///
/// Returns 2-4 lines showing:
/// - Session type (first run vs returning)
/// - Time since last session (if available)
/// - Overall health state
/// - One-sentence summary from daily snapshot or session
///
/// Uses existing data from state, no new RPC calls.
pub fn generate_welcome_lines(state: &AnnaTuiState) -> Vec<Line<'static>> {
    let styles = TuiStyles::default();
    let mut lines = Vec::new();

    // Welcome header
    lines.push(Line::from(vec![
        Span::styled("Welcome to Anna Assistant", styles.bold),
    ]));

    // If brain diagnostics available, show health
    if state.brain_available {
        if let Some(snapshot) = &state.daily_snapshot {
            // Use overall health from snapshot
            if let Some(health) = snapshot.overall_health_opt {
                let health_text = format_health_line(health);
                lines.push(Line::from(vec![
                    Span::styled("Today: ", styles.dimmed),
                    Span::styled(health_text, get_health_style(health)),
                ]));

                // If degraded, mention issue counts
                if health != OverallHealth::Healthy {
                    let issue_text = format!(
                        "{} critical, {} warnings",
                        snapshot.critical_count, snapshot.warning_count
                    );
                    lines.push(Line::from(vec![
                        Span::styled("  ", styles.normal),
                        Span::styled(issue_text, styles.dimmed),
                    ]));
                }
            }
        }
    } else {
        // Daemon unavailable - show fallback
        lines.push(Line::from(vec![
            Span::styled("✗ ", styles.error),
            Span::styled(
                "Brain diagnostics unavailable (daemon offline)",
                styles.dimmed,
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Try: ", styles.dimmed),
            Span::styled("$ sudo systemctl start annad", styles.command),
        ]));
    }

    lines.push(Line::from("")); // Spacing
    lines
}

/// Beta.261: Generate exit summary lines for TUI
///
/// Shows:
/// - Anna version and hostname
/// - Health summary from session (if available)
/// - One diagnostic hint
///
/// Called when user quits TUI, displayed briefly before exit.
pub fn generate_exit_summary(state: &AnnaTuiState) -> Vec<Line<'static>> {
    let styles = TuiStyles::default();
    let mut lines = Vec::new();

    lines.push(Line::from("")); // Top spacing

    // Line 1: Anna version and hostname
    let version = &state.system_panel.anna_version;
    let hostname = if !state.system_panel.hostname.is_empty() {
        &state.system_panel.hostname
    } else {
        "unknown"
    };

    lines.push(Line::from(vec![
        Span::styled("Anna Assistant ", styles.bold),
        Span::styled(format!("v{}", version), styles.dimmed),
        Span::raw(" on "),
        Span::styled(hostname.to_string(), styles.info),
    ]));

    // Line 2: Health summary (matches CLI wording)
    if let Some(snapshot) = &state.daily_snapshot {
        if let Some(health) = snapshot.overall_health_opt {
            let health_text = format_health_line(health);
            lines.push(Line::from(vec![
                Span::styled("System health: ", styles.dimmed),
                Span::styled(health_text, get_health_style(health)),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                "No health data available this session",
                styles.dimmed,
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No health data available this session",
            styles.dimmed,
        )));
    }

    // Line 3: Diagnostic hint (Beta.267: network-aware)
    lines.push(Line::from("")); // Spacing

    // Check for network issues in brain insights
    let has_network_issues = state.brain_insights.iter().any(|i|
        i.rule_id.starts_with("network_") ||
        i.rule_id.contains("packet_loss") ||
        i.rule_id.contains("latency")
    );

    if has_network_issues {
        lines.push(Line::from(vec![
            Span::styled("Tip: ", styles.dimmed),
            Span::raw("Try asking "),
            Span::styled("\"check my network\"", styles.command),
            Span::raw(" for focused network diagnostics"),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("Tip: ", styles.dimmed),
            Span::raw("Try asking "),
            Span::styled("\"check my system health\"", styles.command),
            Span::raw(" for a full diagnostic"),
        ]));
    }

    lines.push(Line::from("")); // Bottom spacing
    lines
}

/// Beta.261: Generate canonical diagnostic hint
///
/// Returns consistent hint used across CLI and TUI:
/// "Try asking: 'check my system health' or 'run a full diagnostic'"
pub fn generate_diagnostic_hint() -> String {
    "Try asking: \"check my system health\" or \"run a full diagnostic\"".to_string()
}

/// Beta.261: Generate daemon unavailable hint for TUI
///
/// Returns lines showing error and recovery command
pub fn generate_daemon_unavailable_lines() -> Vec<Line<'static>> {
    let styles = TuiStyles::default();
    vec![
        Line::from(vec![
            Span::styled("✗ ", styles.error),
            Span::styled("Brain diagnostics unavailable", styles.error),
        ]),
        Line::from(vec![
            Span::styled("  Try: ", styles.dimmed),
            Span::styled("$ sudo systemctl start annad", styles.command),
        ]),
        Line::from(vec![
            Span::styled("  Or: ", styles.dimmed),
            Span::styled("$ annactl status", styles.command),
            Span::styled(" to check daemon health", styles.dimmed),
        ]),
    ]
}

/// Format health state as single line (matches CLI wording from diagnostic_formatter.rs)
fn format_health_line(health: OverallHealth) -> String {
    match health {
        OverallHealth::Healthy => "all clear, no critical issues detected".to_string(),
        OverallHealth::DegradedWarning => "degraded – warning issues detected".to_string(),
        OverallHealth::DegradedCritical => "degraded – critical issues require attention".to_string(),
    }
}

/// Get appropriate style for health state
fn get_health_style(health: OverallHealth) -> Style {
    match health {
        OverallHealth::Healthy => Style::default()
            .fg(Color::Rgb(100, 255, 100))
            .add_modifier(Modifier::BOLD),
        OverallHealth::DegradedWarning => Style::default()
            .fg(Color::Rgb(255, 200, 80))
            .add_modifier(Modifier::BOLD),
        OverallHealth::DegradedCritical => Style::default()
            .fg(Color::Rgb(255, 80, 80))
            .add_modifier(Modifier::BOLD),
    }
}

/// Beta.261: Compute overall health from brain analysis (for testing)
pub fn compute_health_from_analysis(analysis: &BrainAnalysisData) -> OverallHealth {
    compute_overall_health(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic_formatter::DailySnapshotLite;
    use crate::tui_state::AnnaTuiState;

    fn create_test_state_healthy() -> AnnaTuiState {
        let mut state = AnnaTuiState::default();
        state.brain_available = true;
        state.daily_snapshot = Some(DailySnapshotLite {
            overall_health_opt: Some(OverallHealth::Healthy),
            critical_count: 0,
            warning_count: 0,
            kernel_changed: false,
            packages_changed: false,
            boots_since_last: 0,
        });
        state.system_panel.anna_version = "5.7.0-beta.261".to_string();
        state.system_panel.hostname = "testhost".to_string();
        state
    }

    fn create_test_state_degraded_critical() -> AnnaTuiState {
        let mut state = AnnaTuiState::default();
        state.brain_available = true;
        state.daily_snapshot = Some(DailySnapshotLite {
            overall_health_opt: Some(OverallHealth::DegradedCritical),
            critical_count: 2,
            warning_count: 1,
            kernel_changed: false,
            packages_changed: false,
            boots_since_last: 0,
        });
        state.system_panel.anna_version = "5.7.0-beta.261".to_string();
        state.system_panel.hostname = "testhost".to_string();
        state
    }

    fn create_test_state_no_health() -> AnnaTuiState {
        let mut state = AnnaTuiState::default();
        state.brain_available = false;
        state.daily_snapshot = None;
        state.system_panel.anna_version = "5.7.0-beta.261".to_string();
        state.system_panel.hostname = "testhost".to_string();
        state
    }

    #[test]
    fn test_welcome_healthy() {
        let state = create_test_state_healthy();
        let lines = generate_welcome_lines(&state);

        // Should have welcome header, health line, and spacing
        assert!(lines.len() >= 3);

        // First line should be welcome
        let first_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first_text.contains("Welcome"));

        // Should mention "All clear"
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(all_text.contains("All clear"));
    }

    #[test]
    fn test_welcome_degraded_critical() {
        let state = create_test_state_degraded_critical();
        let lines = generate_welcome_lines(&state);

        assert!(lines.len() >= 3);

        // Should mention critical
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(all_text.contains("critical"));
        assert!(all_text.contains("2 critical, 1 warnings"));
    }

    #[test]
    fn test_welcome_no_health() {
        let state = create_test_state_no_health();
        let lines = generate_welcome_lines(&state);

        // Should have fallback message
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(all_text.contains("unavailable"));
        assert!(all_text.contains("systemctl start annad"));
    }

    #[test]
    fn test_exit_summary_healthy() {
        let state = create_test_state_healthy();
        let lines = generate_exit_summary(&state);

        // Should have version, hostname, health, and hint
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(all_text.contains("Anna Assistant"));
        assert!(all_text.contains("v5.7.0-beta.261"));
        assert!(all_text.contains("testhost"));
        assert!(all_text.contains("All clear"));
        assert!(all_text.contains("check my system health"));
    }

    #[test]
    fn test_exit_summary_no_health() {
        let state = create_test_state_no_health();
        let lines = generate_exit_summary(&state);

        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(all_text.contains("No health data available this session"));
        assert!(all_text.contains("check my system health"));
    }

    #[test]
    fn test_diagnostic_hint_consistency() {
        let hint = generate_diagnostic_hint();

        // Should mention both phrases
        assert!(hint.contains("check my system health"));
        assert!(hint.contains("run a full diagnostic"));
    }

    #[test]
    fn test_daemon_unavailable_lines() {
        let lines = generate_daemon_unavailable_lines();

        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        // Should have error marker and recovery commands
        assert!(all_text.contains("✗"));
        assert!(all_text.contains("unavailable"));
        assert!(all_text.contains("systemctl start annad"));
        assert!(all_text.contains("annactl status"));
    }
}
