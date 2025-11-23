//! Utilities - Helper functions for text wrapping, layout, and overlays

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Beta.136: Wrap text to given width, preserving words
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut wrapped = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        // If adding this word would exceed width, start new line
        if !current_line.is_empty() && current_line.len() + 1 + word.len() > width {
            wrapped.push(current_line);
            current_line = word.to_string();
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }

    // Push last line
    if !current_line.is_empty() {
        wrapped.push(current_line);
    }

    // Handle empty input
    if wrapped.is_empty() {
        wrapped.push(String::new());
    }

    wrapped
}

/// Beta.99: Calculate dynamic input bar height
/// Returns height in range [3, 10] based on input content
pub fn calculate_input_height(input: &str, available_width: u16) -> u16 {
    if input.is_empty() {
        return 3; // Minimum height for empty input
    }

    // Account for prompt "[EN] > " (approximately 7 chars) + cursor "_"
    let prompt_width = 8;
    let usable_width = available_width.saturating_sub(prompt_width).max(20) as usize;

    // Calculate how many lines the input will wrap to
    let input_len = input.len();
    let wrapped_lines = input_len.div_ceil(usable_width);

    // Add 2 for borders, then cap between 3 and 10

    (wrapped_lines + 2).max(3).min(10) as u16
}

/// Beta.247: Draw professional help overlay with proper background clearing
/// Fixed: No more text bleed-through from conversation panel
pub fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let help_text = vec![
        // Title
        Line::from(Span::styled(
            "Anna Assistant - Help & Keyboard Shortcuts",
            Style::default()
                .fg(Color::Rgb(255, 200, 80)) // Yellow
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),

        // Section 1: Navigation & Control
        Line::from(Span::styled(
            "Navigation & Control:",
            Style::default()
                .fg(Color::Rgb(100, 200, 255)) // Bright cyan
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Ctrl+C       ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Exit application"),
        ]),
        Line::from(vec![
            Span::styled("  F1           ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Toggle this help overlay"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+L       ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Clear conversation history"),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓        ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Navigate input history"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp / PgDn  ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Scroll conversation"),
        ]),
        Line::from(""),

        // Section 2: Input & Execution
        Line::from(Span::styled(
            "Input & Execution:",
            Style::default()
                .fg(Color::Rgb(100, 200, 255)) // Bright cyan
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Enter        ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Send query to Anna"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+U       ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Clear input buffer"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+X       ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Execute last action plan"),
        ]),
        Line::from(vec![
            Span::styled("  ← / →        ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Move cursor in input"),
        ]),
        Line::from(vec![
            Span::styled("  Home / End   ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" Jump to start/end of input"),
        ]),
        Line::from(""),

        // Section 3: Diagnostic Severity Colors
        Line::from(Span::styled(
            "Diagnostic Severity Colors:",
            Style::default()
                .fg(Color::Rgb(100, 200, 255)) // Bright cyan
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  ✗ ", Style::default().fg(Color::Rgb(255, 80, 80)).add_modifier(Modifier::BOLD)), // Bright red
            Span::styled("Critical  ", Style::default().fg(Color::Rgb(255, 80, 80))), // Bright red
            Span::raw(" Requires immediate attention"),
        ]),
        Line::from(vec![
            Span::styled("  ⚠ ", Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD)), // Yellow
            Span::styled("Warning   ", Style::default().fg(Color::Rgb(255, 200, 80))), // Yellow
            Span::raw(" Should be addressed soon"),
        ]),
        Line::from(vec![
            Span::styled("  ℹ ", Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD)), // Bright cyan
            Span::styled("Info      ", Style::default().fg(Color::Rgb(100, 200, 255))), // Bright cyan
            Span::raw(" Informational, no action needed"),
        ]),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Rgb(100, 255, 100)).add_modifier(Modifier::BOLD)), // Bright green
            Span::styled("Healthy   ", Style::default().fg(Color::Rgb(100, 255, 100))), // Bright green
            Span::raw(" System operating normally"),
        ]),
        Line::from(""),

        // Footer
        Line::from(Span::styled(
            "Press F1 to close  •  Anna v5.7.0-beta.247",
            Style::default().fg(Color::Rgb(120, 120, 120)), // Gray
        )),
    ];

    // Center the help box (60% width, 70% height for more content)
    let help_area = centered_rect(60, 70, area);

    // Beta.247: Render full-screen dimmed overlay first (prevents bleed-through)
    let dim_overlay = Block::default()
        .style(Style::default().bg(Color::Rgb(0, 0, 0))); // Solid black background
    f.render_widget(dim_overlay, area);

    // Then render help box on top with its own background
    let help_block = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 200, 80))), // Yellow
        )
        .style(Style::default().bg(Color::Rgb(20, 20, 20))); // Darker background for contrast

    f.render_widget(help_block, help_area);
}

/// Create a centered rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
