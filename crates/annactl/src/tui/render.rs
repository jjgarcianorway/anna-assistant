//! Rendering - UI drawing functions for conversation, header, and status bar
//! Beta.260: Now uses canonical answer formatting via formatting module
//! Beta.262: Uses centralized layout grid from layout module

use crate::tui_state::{AnnaTuiState, ChatItem};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::action_plan::render_action_plan_lines;
use super::formatting::{parse_canonical_format, TuiStyles};
use super::input::draw_input_bar;
use super::layout;
use super::utils::{draw_help_overlay, wrap_text};

/// Draw the UI - Claude CLI style with header and status bar
/// Beta.218: Added brain diagnostics panel on the right
/// Beta.262: Uses centralized layout grid computation
pub fn draw_ui(f: &mut Frame, state: &AnnaTuiState) {
    let size = f.size();

    // Beta.262: Compute canonical layout grid
    let layout_grid = layout::compute_layout(size);

    // Draw top header
    draw_header(f, layout_grid.header, state);

    // Beta.262: Only split content if diagnostics panel has height
    if layout_grid.diagnostics.height > 0 {
        // Split conversation area horizontally: [Conversation | Brain Panel]
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65), // Conversation panel (left, 65%)
                Constraint::Percentage(35), // Brain panel (right, 35%)
            ])
            .split(layout_grid.conversation);

        // Draw conversation panel (left side)
        draw_conversation_panel(f, content_chunks[0], state);

        // Beta.218: Draw brain diagnostics panel (right side)
        super::brain::draw_brain_panel(f, content_chunks[1], state);
    } else {
        // Small terminal - conversation takes full width, no diagnostics panel
        draw_conversation_panel(f, layout_grid.conversation, state);
    }

    // Draw bottom status bar
    draw_status_bar(f, layout_grid.status_bar, state);

    // Draw input bar
    draw_input_bar(f, layout_grid.input, state);

    // Draw help overlay if active
    if state.show_help {
        draw_help_overlay(f, size);
    }
}

/// Draw professional header (Claude CLI style)
/// Beta.230: Simplified to avoid redundancy with status bar - shows context not status
/// Beta.260: Full-width, single clean bar with consistent styling
/// Beta.262: Uses layout::compose_header_text for deterministic truncation
/// Format: Anna v5.7.0-beta.262 | llama3.2:3b | user@hostname
pub fn draw_header(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use std::env;

    let hostname = if !state.system_panel.hostname.is_empty() {
        state.system_panel.hostname.as_str()
    } else {
        "unknown"
    };
    let username = env::var("USER").unwrap_or_else(|_| "user".to_string());

    // Beta.262: Use composition function for truncation logic
    let text = layout::compose_header_text(
        area.width,
        &state.system_panel.anna_version,
        &username,
        hostname,
        &state.llm_panel.model_name,
    );

    // Build styled spans from composed text
    // For simplicity, use uniform styling for the entire header
    let header_text = Line::from(vec![
        Span::raw(" "), // Left padding for visual breathing room
        Span::styled(
            text,
            Style::default().fg(Color::Rgb(150, 200, 255)), // Bright cyan/blue
        ),
    ]);

    // Beta.260: Full-width header with consistent background
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::Rgb(0, 0, 0)));

    f.render_widget(header, area);
}

/// Draw professional status bar (bottom)
/// Beta.230: Streamlined - removed redundant hostname and LLM status, focused on real-time metrics
/// Beta.247: Verified high-res layout sanity - spans build left-to-right, truncation handled by ratatui
/// Beta.262: Uses layout::compose_status_bar_text for deterministic truncation
/// Format: Mode: TUI | 15:42:08 | Health: ✓ | CPU: 8% | RAM: 4.2GB | Daemon: ✓ | Brain: 2⚠
pub fn draw_status_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use chrono::Local;

    let now = Local::now();
    let time_str = now.format("%H:%M:%S").to_string();

    // Count brain issues for composition function
    let critical_count = state
        .brain_insights
        .iter()
        .filter(|i| i.severity.to_lowercase() == "critical")
        .count();
    let warning_count = state
        .brain_insights
        .iter()
        .filter(|i| i.severity.to_lowercase() == "warning")
        .count();

    // Determine health status
    let health_ok = if state.brain_available {
        critical_count == 0 && warning_count == 0
    } else {
        state.system_panel.cpu_load_1min < 80.0 && state.system_panel.ram_used_gb < 14.0
    };

    let daemon_ok = state.telemetry_ok || state.brain_available;

    // Beta.262: Use composition function for truncation logic
    let text = layout::compose_status_bar_text(
        area.width,
        &time_str,
        state.is_thinking,
        health_ok,
        state.system_panel.cpu_load_1min,
        state.system_panel.ram_used_gb,
        daemon_ok,
        critical_count,
        warning_count,
    );

    // Build styled line from composed text
    let status_text = Line::from(vec![
        Span::raw(" "), // Left padding
        Span::styled(
            text,
            Style::default().fg(Color::Rgb(180, 180, 180)), // Light gray
        ),
    ]);

    // Beta.220: Use truecolor background for status bar
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));

    f.render_widget(status_bar, area);
}

/// Draw conversation panel (middle)
/// Beta.99: Added scrolling support with PageUp/PageDown
/// Beta.136: Fixed scroll calculation to account for text wrapping
pub fn draw_conversation_panel(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    // Beta.136: Calculate available width for text (subtract borders and padding)
    let content_width = area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding
    let mut lines: Vec<Line<'static>> = Vec::new();

    for item in &state.conversation {
        match item {
            ChatItem::User(msg) => {
                // Beta.136: Wrap user messages manually to get accurate line count
                let prefix = "You: ";
                let wrapped = wrap_text(msg, content_width.saturating_sub(prefix.len()));
                for (i, wrapped_line) in wrapped.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                prefix,
                                Style::default()
                                    .fg(Color::Rgb(100, 150, 255)) // Blue
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(wrapped_line.clone()),
                        ]));
                    } else {
                        // Indent continuation lines
                        lines.push(Line::from(Span::raw(format!("     {}", wrapped_line))));
                    }
                }
                lines.push(Line::from("")); // Add spacing between messages
            }
            ChatItem::Anna(msg) => {
                // Beta.260: Use canonical formatting for Anna's responses
                lines.push(Line::from(vec![Span::styled(
                    "Anna: ",
                    Style::default()
                        .fg(Color::Rgb(100, 255, 100)) // Bright green
                        .add_modifier(Modifier::BOLD),
                )]));

                // Parse message using canonical format parser
                let styles = TuiStyles::default();
                let formatted_lines = parse_canonical_format(msg, &styles);

                // Add formatted lines with wrapping
                for formatted_line in formatted_lines {
                    // Check if line needs wrapping
                    let line_text: String = formatted_line.spans.iter().map(|s| s.content.as_ref()).collect();

                    if line_text.len() > content_width {
                        // Need to wrap - preserve styling for first segment
                        let wrapped = wrap_text(&line_text, content_width);
                        for (i, wrapped_line) in wrapped.iter().enumerate() {
                            if i == 0 && !formatted_line.spans.is_empty() {
                                // Use original styling for first line
                                lines.push(formatted_line.clone());
                            } else {
                                // Continuation lines in normal style
                                lines.push(Line::from(Span::raw(wrapped_line.clone())));
                            }
                        }
                    } else {
                        // No wrapping needed, use as-is
                        lines.push(formatted_line);
                    }
                }
                lines.push(Line::from("")); // Add spacing between messages
            }
            ChatItem::System(msg) => {
                // Beta.136: Wrap system messages
                let prefix = "System: ";
                let wrapped = wrap_text(msg, content_width.saturating_sub(prefix.len()));
                for (i, wrapped_line) in wrapped.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(Color::Rgb(255, 200, 80))), // Yellow
                            Span::raw(wrapped_line.clone()),
                        ]));
                    } else {
                        lines.push(Line::from(Span::raw(format!("        {}", wrapped_line))));
                    }
                }
                lines.push(Line::from("")); // Add spacing between messages
            }
            ChatItem::ActionPlan(plan) => {
                // Beta.147: Render structured action plan
                render_action_plan_lines(&mut lines, plan, content_width);
                lines.push(Line::from("")); // Add spacing
            }
        }
    }

    // Beta.136: Now total_lines accurately reflects wrapped content
    let total_lines = lines.len();
    let visible_lines = area.height.saturating_sub(2) as usize; // Subtract 2 for borders
    let max_scroll = total_lines.saturating_sub(visible_lines);

    // Beta.115: Auto-scroll to bottom if scroll_offset is at max
    let actual_scroll = if state.scroll_offset == usize::MAX || state.scroll_offset >= max_scroll {
        max_scroll
    } else {
        state.scroll_offset
    };

    // Beta.262: Use scroll indicator helpers from layout module
    let can_scroll_up = layout::should_show_scroll_up_indicator(actual_scroll);
    let can_scroll_down = layout::should_show_scroll_down_indicator(total_lines, visible_lines, actual_scroll);

    // Build title with scroll indicators
    let scroll_indicator = if total_lines > visible_lines {
        let up_arrow = if can_scroll_up { "▲" } else { " " };
        let down_arrow = if can_scroll_down { "▼" } else { " " };
        format!(" {}{} ", up_arrow, down_arrow)
    } else {
        String::new()
    };

    // Beta.262: Consistent panel border style with scroll indicators
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" Conversation{} ", scroll_indicator))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(80, 180, 255))), // Bright cyan
        )
        // Beta.136: Disable auto-wrap (we wrap manually now for accurate scroll)
        .scroll((actual_scroll as u16, 0)); // Beta.99: Enable scrolling!

    f.render_widget(paragraph, area);
}
