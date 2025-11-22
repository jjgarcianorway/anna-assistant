//! Brain Diagnostics Panel - Sysadmin Brain integration for TUI
//!
//! Beta.218: Displays top 3 system insights with severity indicators

use crate::rpc_client::RpcClient;
use crate::tui_state::AnnaTuiState;
use anna_common::ipc::{BrainAnalysisData, Method, ResponseData};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Fetch brain analysis from daemon and update state
pub async fn update_brain_analysis(state: &mut AnnaTuiState) {
    match fetch_brain_data().await {
        Ok(analysis) => {
            // Take top 3 insights (sorted by severity: critical > warning > info)
            let mut insights = analysis.insights;
            insights.sort_by(|a, b| {
                let a_priority = severity_priority(&a.severity);
                let b_priority = severity_priority(&b.severity);
                b_priority.cmp(&a_priority) // Descending order (higher priority first)
            });
            insights.truncate(3);

            state.brain_insights = insights;
            state.brain_timestamp = Some(analysis.timestamp);
            state.brain_available = true;
        }
        Err(_) => {
            // Daemon unavailable - graceful fallback
            state.brain_insights.clear();
            state.brain_timestamp = None;
            state.brain_available = false;
        }
    }
}

/// Fetch brain analysis via RPC
/// Beta.234: Made public for background task usage
pub async fn fetch_brain_data() -> anyhow::Result<BrainAnalysisData> {
    let mut client = RpcClient::connect_quick(None).await?;
    let response = client.call(Method::BrainAnalysis).await?;

    match response {
        ResponseData::BrainAnalysis(data) => Ok(data),
        _ => Err(anyhow::anyhow!("Unexpected response type")),
    }
}

/// Get severity priority for sorting (higher = more important)
fn severity_priority(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "critical" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}

/// Draw brain diagnostics panel (right side)
pub fn draw_brain_panel(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    if !state.brain_available {
        // Fallback: Daemon unavailable
        draw_brain_fallback(f, area);
        return;
    }

    if state.brain_insights.is_empty() {
        // All systems healthy
        draw_brain_healthy(f, area);
        return;
    }

    // Render insights
    let mut lines: Vec<Line> = Vec::new();

    for (i, insight) in state.brain_insights.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from("")); // Spacing between insights
        }

        // Severity indicator and summary
        let (emoji, color) = match insight.severity.to_lowercase().as_str() {
            "critical" => ("✗", Color::Rgb(255, 80, 80)), // Bright red
            "warning" => ("⚠", Color::Rgb(255, 200, 80)), // Yellow
            "info" => ("ℹ", Color::Rgb(100, 200, 255)), // Bright cyan
            _ => ("•", Color::Rgb(120, 120, 120)), // Gray
        };

        lines.push(Line::from(vec![
            Span::styled(emoji.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(
                insight.summary.clone(),
                Style::default().fg(Color::Rgb(240, 240, 240)).add_modifier(Modifier::BOLD), // Near-white
            ),
        ]));

        // Evidence (if available and not empty)
        if !insight.evidence.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Evidence: ".to_string(), Style::default().fg(Color::Rgb(120, 120, 120))), // Gray
                Span::raw(insight.evidence.clone()),
            ]));
        }

        // Commands (first command only for space efficiency)
        if let Some(cmd) = insight.commands.first() {
            lines.push(Line::from(vec![
                Span::styled("  Fix: ".to_string(), Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
                Span::styled(cmd.clone(), Style::default().fg(Color::Rgb(100, 200, 255))), // Bright cyan
            ]));
        }
    }

    let title = format!(
        " Brain Diagnostics ({} insights) ",
        state.brain_insights.len()
    );

    // Beta.220: Truecolor border based on severity
    let border_color = if state.brain_insights.iter().any(|i| i.severity.to_lowercase() == "critical") {
        Color::Rgb(255, 80, 80) // Bright red for critical
    } else if state.brain_insights.iter().any(|i| i.severity.to_lowercase() == "warning") {
        Color::Rgb(255, 180, 0) // Orange for warnings
    } else {
        Color::Rgb(100, 200, 100) // Green for healthy
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Draw fallback message when daemon unavailable
fn draw_brain_fallback(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Brain diagnostics unavailable",
            Style::default().fg(Color::Rgb(255, 200, 80)), // Yellow
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Ensure annad daemon is running:",
            Style::default().fg(Color::Rgb(120, 120, 120)), // Gray
        )]),
        Line::from(vec![Span::styled(
            "  sudo systemctl start annad",
            Style::default().fg(Color::Rgb(100, 200, 255)), // Bright cyan
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Brain Diagnostics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 180, 0))), // Orange for unavailable
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Draw healthy state message
fn draw_brain_healthy(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("✓", Style::default().fg(Color::Rgb(100, 255, 100)).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(
                "All systems healthy",
                Style::default().fg(Color::Rgb(100, 255, 100)).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "No issues detected by diagnostic rules.",
            Style::default().fg(Color::Rgb(150, 150, 150)),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Brain Diagnostics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(100, 200, 100))), // Green for healthy
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
