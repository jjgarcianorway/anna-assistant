//! TUI Text Formatting Module (Beta.260)
//!
//! Provides semantic text parsing and Ratatui styling for canonical answer format.
//!
//! Philosophy:
//! - Same semantic structure as CLI (defined in ANSWER_FORMAT.md)
//! - Convert [SUMMARY]/[DETAILS]/[COMMANDS] sections to visual hierarchy
//! - Convert **bold** markdown to Ratatui bold style
//! - Highlight command lines ($ and # prefixed)
//! - Handle error/warning/info markers (✗, ⚠, ℹ) with appropriate colors
//!
//! This module ensures TUI and CLI look like "different views of the same answer."

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Beta.260: TUI Style Map - Consistent visual hierarchy
///
/// This defines the semantic styles used throughout the TUI to ensure
/// consistent presentation of the canonical answer format.
pub struct TuiStyles {
    /// Section headers like [SUMMARY], [DETAILS], [COMMANDS]
    pub section_header: Style,
    /// Command lines starting with $ or #
    pub command: Style,
    /// Bold text (from **bold** markdown)
    pub bold: Style,
    /// Error markers (✗) and critical text
    pub error: Style,
    /// Warning markers (⚠) and warning text
    pub warning: Style,
    /// Info markers (ℹ) and info text
    pub info: Style,
    /// Success markers (✓)
    pub success: Style,
    /// Normal text
    pub normal: Style,
    /// Dimmed/secondary text
    pub dimmed: Style,
}

impl Default for TuiStyles {
    fn default() -> Self {
        Self {
            // Section headers: uppercase, bright cyan, bold
            section_header: Style::default()
                .fg(Color::Rgb(80, 180, 255))
                .add_modifier(Modifier::BOLD),

            // Commands: bright green
            command: Style::default().fg(Color::Rgb(100, 255, 100)),

            // Bold text: white, bold
            bold: Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .add_modifier(Modifier::BOLD),

            // Error: bright red, bold
            error: Style::default()
                .fg(Color::Rgb(255, 80, 80))
                .add_modifier(Modifier::BOLD),

            // Warning: yellow/orange, bold
            warning: Style::default()
                .fg(Color::Rgb(255, 200, 80))
                .add_modifier(Modifier::BOLD),

            // Info: bright cyan
            info: Style::default().fg(Color::Rgb(100, 200, 255)),

            // Success: bright green, bold
            success: Style::default()
                .fg(Color::Rgb(100, 255, 100))
                .add_modifier(Modifier::BOLD),

            // Normal: light gray
            normal: Style::default().fg(Color::Rgb(200, 200, 200)),

            // Dimmed: darker gray
            dimmed: Style::default().fg(Color::Rgb(120, 120, 120)),
        }
    }
}

/// Beta.260: Parse canonical answer format into styled Ratatui lines
///
/// Takes text in canonical [SUMMARY]/[DETAILS]/[COMMANDS] format and returns
/// a vector of Ratatui Line objects with appropriate styling applied.
///
/// Features:
/// - Section headers ([SUMMARY], etc.) styled as headers
/// - **bold** markdown converted to bold style
/// - Command lines ($, #) highlighted
/// - Error/warning/info markers (✗, ⚠, ℹ) colored appropriately
/// - Preserves line breaks and indentation
pub fn parse_canonical_format(text: &str, styles: &TuiStyles) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Handle code block markers (```)
        if trimmed == "```bash" || trimmed == "```" {
            in_code_block = !in_code_block;
            continue; // Skip the marker line
        }

        // Section headers: [SUMMARY], [DETAILS], [COMMANDS], etc.
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            lines.push(Line::from(vec![
                Span::styled(" ", styles.normal), // Add slight padding
                Span::styled(trimmed.to_string(), styles.section_header),
            ]));
        }
        // Command lines (starting with $ or #) or inside code blocks
        else if trimmed.starts_with('$') || trimmed.starts_with('#') || in_code_block {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                styles.command,
            )));
        }
        // Regular content - parse for bold, markers, etc.
        else {
            lines.push(parse_line_with_formatting(line, styles));
        }
    }

    lines
}

/// Parse a single line and apply inline formatting (bold, markers)
fn parse_line_with_formatting(text: &str, styles: &TuiStyles) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current_text = String::new();
    let mut in_bold = false;

    while let Some(ch) = chars.next() {
        // Check for ** (bold marker)
        if ch == '*' && chars.peek() == Some(&'*') {
            // Flush current text
            if !current_text.is_empty() {
                let style = if in_bold { styles.bold } else { styles.normal };
                spans.push(Span::styled(current_text.clone(), style));
                current_text.clear();
            }

            // Consume second *
            chars.next();

            // Toggle bold state
            in_bold = !in_bold;
        }
        // Check for special markers at start of accumulation
        else if current_text.is_empty() && is_marker_char(ch) {
            // Look ahead to see if this is a standalone marker
            let marker = ch.to_string();
            current_text.push(ch);

            // Check if next char is space (standalone marker)
            if chars.peek() == Some(&' ') || chars.peek().is_none() {
                let marker_style = get_marker_style(&marker, styles);
                spans.push(Span::styled(current_text.clone(), marker_style));
                current_text.clear();
            }
        } else {
            current_text.push(ch);
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        let style = if in_bold { styles.bold } else { styles.normal };
        spans.push(Span::styled(current_text, style));
    }

    // If no spans were created, return empty line with normal style
    if spans.is_empty() {
        Line::from(Span::styled("".to_string(), styles.normal))
    } else {
        Line::from(spans)
    }
}

/// Check if character is a special marker (✗, ⚠, ℹ, ✓)
fn is_marker_char(ch: char) -> bool {
    matches!(ch, '✗' | '⚠' | 'ℹ' | '✓' | '→')
}

/// Get appropriate style for a marker character
fn get_marker_style(marker: &str, styles: &TuiStyles) -> Style {
    match marker {
        "✗" => styles.error,
        "⚠" => styles.warning,
        "ℹ" => styles.info,
        "✓" => styles.success,
        "→" => styles.info,
        _ => styles.normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_section_headers() {
        let styles = TuiStyles::default();
        let text = "[SUMMARY]\nTest content";
        let lines = parse_canonical_format(text, &styles);

        assert_eq!(lines.len(), 2);
        // First line should be section header
        assert!(lines[0].spans[1].content.contains("[SUMMARY]"));
    }

    #[test]
    fn test_parse_bold_formatting() {
        let styles = TuiStyles::default();
        let text = "This is **bold** text";
        let lines = parse_canonical_format(text, &styles);

        assert_eq!(lines.len(), 1);
        // Should have multiple spans (normal, bold, normal)
        assert!(lines[0].spans.len() >= 2);
    }

    #[test]
    fn test_parse_command_lines() {
        let styles = TuiStyles::default();
        let text = "$ systemctl status annad\n# This is a comment";
        let lines = parse_canonical_format(text, &styles);

        assert_eq!(lines.len(), 2);
        // Both should be styled as commands
        assert!(lines[0].spans[0].content.starts_with('$'));
        assert!(lines[1].spans[0].content.starts_with('#'));
    }

    #[test]
    fn test_parse_markers() {
        let styles = TuiStyles::default();
        let text = "✗ Error occurred\n⚠ Warning message\n✓ Success";
        let lines = parse_canonical_format(text, &styles);

        assert_eq!(lines.len(), 3);
        // Each line should start with a marker
        assert!(lines[0].spans[0].content.contains('✗'));
        assert!(lines[1].spans[0].content.contains('⚠'));
        assert!(lines[2].spans[0].content.contains('✓'));
    }

    #[test]
    fn test_code_blocks_stripped() {
        let styles = TuiStyles::default();
        let text = "```bash\n$ command\n```";
        let lines = parse_canonical_format(text, &styles);

        // Should have 1 line (the command), not 3
        assert_eq!(lines.len(), 1);
        assert!(lines[0].spans[0].content.contains("$ command"));
    }

    #[test]
    fn test_nested_bold_and_markers() {
        let styles = TuiStyles::default();
        let text = "System health **degraded – critical issues**";
        let lines = parse_canonical_format(text, &styles);

        assert_eq!(lines.len(), 1);
        // Should have spans for normal + bold + normal
        assert!(lines[0].spans.len() >= 2);
    }
}
