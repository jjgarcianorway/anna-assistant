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

/// Draw help overlay
pub fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::raw(" - Exit"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+L", Style::default().fg(Color::Cyan)),
            Span::raw(" - Clear conversation"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
            Span::raw(" - Clear input"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+X", Style::default().fg(Color::Cyan)),
            Span::raw(" - Execute action plan"),
        ]),
        Line::from(vec![
            Span::styled("F1", Style::default().fg(Color::Cyan)),
            Span::raw(" - Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" - Navigate history"),
        ]),
        Line::from(vec![
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan)),
            Span::raw(" - Scroll conversation"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press F1 to close",
            Style::default().fg(Color::Gray),
        )),
    ];

    // Center the help box
    let help_area = centered_rect(50, 50, area);

    let help_block = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().bg(Color::Black));

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
