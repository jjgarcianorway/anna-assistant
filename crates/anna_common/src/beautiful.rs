//! Beautiful terminal output using proper libraries
//!
//! Now using owo-colors and console for robust, battle-tested formatting!

use console::{measure_text_width, Term};
use owo_colors::OwoColorize;

/// Status level for messages
#[derive(Debug, Clone, Copy)]
pub enum Level {
    Info,
    Success,
    Warning,
    Error,
}

impl Level {
    pub fn symbol(&self) -> &'static str {
        match self {
            Level::Info => "ℹ",
            Level::Success => "✓",
            Level::Warning => "⚠",
            Level::Error => "✗",
        }
    }
}

/// Format a header with beautiful box drawing
pub fn header(text: &str) -> String {
    // Measure the VISIBLE width (without ANSI codes)
    let text_width = measure_text_width(text);
    let padding = 3; // More space on each side for better readability
    let inner_width = text_width + (padding * 2);

    // Build the box components as strings
    let top_line = format!("╭{}╮", "─".repeat(inner_width));
    let bottom_line = format!("╰{}╯", "─".repeat(inner_width));

    // Build padding strings
    let left_padding = " ".repeat(padding);
    let right_padding = " ".repeat(padding);

    // Apply colors using format! to avoid lifetime issues
    let colored_top = format!("{}", top_line.cyan());
    let colored_middle = format!(
        "{}{}{}{}{}",
        "│".cyan(),
        left_padding,
        text.bold().bright_cyan(),
        right_padding,
        "│".cyan()
    );
    let colored_bottom = format!("{}", bottom_line.cyan());

    format!(
        "\n{}\n{}\n{}\n",
        colored_top, colored_middle, colored_bottom
    )
}

/// Format a section title
pub fn section(text: &str) -> String {
    format!("{} {}", "→".cyan().bold(), text.bold().cyan())
}

/// Format a status message
pub fn status(level: Level, message: &str) -> String {
    let symbol = level.symbol();
    match level {
        Level::Info => format!("{} {}", symbol.cyan(), message),
        Level::Success => format!("{} {}", symbol.green(), message),
        Level::Warning => format!("{} {}", symbol.yellow(), message),
        Level::Error => format!("{} {}", symbol.red(), message),
    }
}

/// Format a key-value pair
pub fn kv(key: &str, value: &str) -> String {
    format!("{}: {}", key.bright_black(), value)
}

/// Format a box around text - DEPRECATED, use simple formatting instead
#[deprecated(note = "Use simple formatting instead of boxes")]
pub fn boxed(lines: &[&str]) -> String {
    // Just return lines with simple indentation
    lines
        .iter()
        .map(|line| format!("   {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get terminal width for responsive formatting
pub fn terminal_width() -> usize {
    Term::stdout().size().1 as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::strip_ansi_codes;

    #[test]
    fn test_header_formatting() {
        let h = header("Test Header");
        // Strip ANSI codes to test the actual box structure
        let stripped = strip_ansi_codes(&h);
        assert!(stripped.contains("╭"));
        assert!(stripped.contains("╯"));
        assert!(stripped.contains("│"));
        assert!(stripped.contains("Test Header"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_box_with_ansi() {
        let lines = vec!["Hello", "World"];
        let boxed = boxed(&lines);
        assert!(boxed.contains("Hello"));
        assert!(boxed.contains("World"));
    }
}
