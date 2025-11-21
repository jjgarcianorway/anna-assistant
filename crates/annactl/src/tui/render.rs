//! Rendering - UI drawing functions for conversation, header, and status bar

use crate::tui_state::{AnnaTuiState, ChatItem};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::action_plan::render_action_plan_lines;
use super::input::draw_input_bar;
use super::utils::{calculate_input_height, draw_help_overlay, wrap_text};

/// Draw the UI - Claude CLI style with header and status bar
/// Beta.218: Added brain diagnostics panel on the right
pub fn draw_ui(f: &mut Frame, state: &AnnaTuiState) {
    let size = f.size();

    // Beta.99: Calculate dynamic input bar height (3 to 10 lines max)
    let input_height = calculate_input_height(&state.input, size.width.saturating_sub(8));

    // Create main layout: [Header | Content | Status Bar | Input Bar]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),            // Top header
            Constraint::Min(3),               // Main content
            Constraint::Length(1),            // Bottom status bar
            Constraint::Length(input_height), // Beta.99: Dynamic input bar
        ])
        .split(size);

    // Draw top header
    draw_header(f, main_chunks[0], state);

    // Beta.218: Split content area: [Conversation | Brain Panel]
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Conversation panel (left, 65%)
            Constraint::Percentage(35), // Brain panel (right, 35%)
        ])
        .split(main_chunks[1]);

    // Draw conversation panel (left side)
    draw_conversation_panel(f, content_chunks[0], state);

    // Beta.218: Draw brain diagnostics panel (right side)
    super::brain::draw_brain_panel(f, content_chunks[1], state);

    // Draw bottom status bar
    draw_status_bar(f, main_chunks[2], state);

    // Draw input bar
    draw_input_bar(f, main_chunks[3], state);

    // Draw help overlay if active
    if state.show_help {
        draw_help_overlay(f, size);
    }
}

/// Draw professional header (Claude CLI style)
/// Beta.230: Simplified to avoid redundancy with status bar - shows context not status
/// Format: Anna v5.7.0-beta.230 | llama3.2:3b | user@hostname
pub fn draw_header(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use std::env;

    let hostname = if !state.system_panel.hostname.is_empty() {
        state.system_panel.hostname.as_str()
    } else {
        "unknown"
    };
    let username = env::var("USER").unwrap_or_else(|_| "user".to_string());

    let header_text = Line::from(vec![
        Span::styled(
            "Anna ",
            Style::default()
                .fg(Color::Rgb(100, 200, 255)) // Bright cyan
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("v{}", state.system_panel.anna_version),
            Style::default().fg(Color::Rgb(120, 120, 120)), // Gray
        ),
        Span::raw(" │ "),
        Span::styled(
            &state.llm_panel.model_name,
            Style::default().fg(Color::Rgb(255, 200, 80)), // Yellow
        ),
        Span::raw(" │ "),
        Span::styled(
            format!("{}@{}", username, hostname),
            Style::default().fg(Color::Rgb(100, 150, 255)), // Blue
        ),
    ]);

    let header = Paragraph::new(header_text).style(Style::default().bg(Color::Rgb(0, 0, 0)));

    f.render_widget(header, area);
}

/// Draw professional status bar (bottom)
/// Beta.230: Streamlined - removed redundant hostname and LLM status, focused on real-time metrics
/// Format: Mode: TUI | 15:42:08 | Health: ✓ | CPU: 8% | RAM: 4.2GB | Daemon: ✓ | Brain: 2⚠
pub fn draw_status_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use chrono::Local;

    let now = Local::now();
    let time_str = now.format("%H:%M:%S").to_string();

    // Beta.91: Thinking indicator with animation
    let thinking_spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let thinking_indicator = if state.is_thinking {
        let frame = thinking_spinner[state.thinking_frame % thinking_spinner.len()];
        Some((frame, "Thinking..."))
    } else {
        None
    };

    // Beta.218: Enhanced health check including brain diagnostics
    let health_icon = if state.brain_available {
        // Count critical issues from brain
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

        if critical_count > 0 {
            ("✗", Color::Rgb(255, 80, 80)) // Bright red
        } else if warning_count > 0 {
            ("⚠", Color::Rgb(255, 200, 80)) // Yellow/orange
        } else {
            ("✓", Color::Rgb(100, 255, 100)) // Bright green
        }
    } else {
        // Fallback to CPU/RAM check if brain unavailable
        if state.system_panel.cpu_load_1min < 80.0 && state.system_panel.ram_used_gb < 14.0 {
            ("✓", Color::Rgb(100, 255, 100)) // Bright green
        } else if state.system_panel.cpu_load_1min < 95.0 {
            ("⚠", Color::Rgb(255, 200, 80)) // Yellow/orange
        } else {
            ("✗", Color::Rgb(255, 80, 80)) // Bright red
        }
    };

    // Beta.230: Build status bar with consistent sections (no redundant hostname)
    let mut spans = Vec::new();

    // Section 2: Mode
    spans.push(Span::styled("Mode: ", Style::default().fg(Color::Rgb(120, 120, 120)))); // Gray
    spans.push(Span::styled(
        "TUI",
        Style::default().fg(Color::Rgb(100, 255, 100)), // Bright green
    ));
    spans.push(Span::raw(" │ "));

    // Section 3: Clock (truecolor)
    spans.push(Span::styled(
        time_str,
        Style::default().fg(Color::Rgb(150, 150, 150)),
    ));
    spans.push(Span::raw(" │ "));

    // Section 4: Thinking indicator or Health
    if let Some((spinner, text)) = thinking_indicator {
        spans.push(Span::styled(spinner, Style::default().fg(Color::Rgb(100, 200, 255)))); // Bright cyan
        spans.push(Span::raw(" "));
        spans.push(Span::styled(text, Style::default().fg(Color::Rgb(100, 200, 255)))); // Bright cyan
    } else {
        spans.push(Span::styled("Health: ", Style::default().fg(Color::Rgb(120, 120, 120)))); // Gray
        spans.push(Span::styled(
            health_icon.0,
            Style::default().fg(health_icon.1).add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::raw(" │ "));

    // Section 5: CPU and RAM (compact)
    spans.push(Span::styled(
        format!("CPU: {:.0}%", state.system_panel.cpu_load_1min),
        Style::default().fg(Color::Rgb(180, 180, 180)),
    ));
    spans.push(Span::raw(" │ "));
    spans.push(Span::styled(
        format!("RAM: {:.1}G", state.system_panel.ram_used_gb),
        Style::default().fg(Color::Rgb(180, 180, 180)),
    ));
    spans.push(Span::raw(" │ "));

    // Section 6: Daemon Status (based on telemetry_ok and brain_available)
    let daemon_ok = state.telemetry_ok || state.brain_available;
    spans.push(Span::styled("Daemon: ", Style::default().fg(Color::Rgb(120, 120, 120)))); // Gray
    spans.push(Span::styled(
        if daemon_ok { "✓" } else { "✗" },
        Style::default().fg(if daemon_ok {
            Color::Rgb(100, 255, 100) // Bright green
        } else {
            Color::Rgb(255, 80, 80) // Bright red
        }).add_modifier(Modifier::BOLD),
    ));

    // Section 7: Brain diagnostics (only if issues exist)
    if state.brain_available && !state.brain_insights.is_empty() {
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

        if critical_count > 0 {
            spans.push(Span::raw(" │ "));
            spans.push(Span::styled("Brain: ", Style::default().fg(Color::Rgb(120, 120, 120)))); // Gray
            spans.push(Span::styled(
                format!("{}✗", critical_count),
                Style::default().fg(Color::Rgb(255, 80, 80)).add_modifier(Modifier::BOLD), // Bright red
            ));
        } else if warning_count > 0 {
            spans.push(Span::raw(" │ "));
            spans.push(Span::styled("Brain: ", Style::default().fg(Color::Rgb(120, 120, 120)))); // Gray
            spans.push(Span::styled(
                format!("{}⚠", warning_count),
                Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD), // Yellow
            ));
        }
    }

    // Beta.220: Use truecolor background for status bar
    let status_text = Line::from(spans);
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
                lines.push(Line::from(vec![Span::styled(
                    "Anna: ",
                    Style::default()
                        .fg(Color::Rgb(100, 255, 100)) // Bright green
                        .add_modifier(Modifier::BOLD),
                )]));
                // Beta.136: Wrap each line of Anna's reply
                for line in msg.lines() {
                    let wrapped = wrap_text(line, content_width);
                    for wrapped_line in wrapped {
                        lines.push(Line::from(Span::raw(wrapped_line)));
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

    let scroll_indicator = if total_lines > visible_lines {
        format!(" [↑↓ {}/{}]", actual_scroll, max_scroll)
    } else {
        String::new()
    };

    // Beta.220: Truecolor conversation panel border
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
