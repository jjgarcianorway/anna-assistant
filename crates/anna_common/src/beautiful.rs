//! Beautiful terminal output using proper libraries
//!
//! Now using owo-colors and console for robust, battle-tested formatting!

use owo_colors::OwoColorize;
use console::{measure_text_width, Term};

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

/// Format a header with proper box drawing
pub fn header(text: &str) -> String {
    let visible_width = measure_text_width(text);
    let padding = 2;
    let total_width = visible_width + (padding * 2);

    let top = format!("╭{}╮", "─".repeat(total_width));
    let middle = format!("│ {} │", text);
    let bottom = format!("╰{}╯", "─".repeat(total_width));

    format!(
        "{}\n{}\n{}",
        top.bold().blue(),
        middle.bold().blue(),
        bottom.bold().blue()
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

/// Format a box around text with proper width calculation
pub fn boxed(lines: &[&str]) -> String {
    // Calculate max visible width (console handles all unicode properly)
    let max_width = lines.iter()
        .map(|line| measure_text_width(line))
        .max()
        .unwrap_or(0);

    let top = format!("╭{}╮", "─".repeat(max_width + 2));
    let bottom = format!("╰{}╯", "─".repeat(max_width + 2));

    let mut result = vec![top.blue().to_string()];

    for line in lines {
        let visible_width = measure_text_width(line);
        let padding_needed = max_width - visible_width;
        let padding = " ".repeat(padding_needed);
        result.push(format!("{} {}{} {}",
            "│".blue(),
            line,
            padding,
            "│".blue()
        ));
    }

    result.push(bottom.blue().to_string());
    result.join("\n")
}

/// Get terminal width for responsive formatting
pub fn terminal_width() -> usize {
    Term::stdout().size().1 as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_formatting() {
        let h = header("Test Header");
        assert!(h.contains("╭"));
        assert!(h.contains("╯"));
    }

    #[test]
    fn test_box_with_ansi() {
        let lines = vec!["Hello", "World"];
        let boxed = boxed(&lines);
        assert!(boxed.contains("│"));
    }
}
