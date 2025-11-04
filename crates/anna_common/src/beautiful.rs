//! Beautiful terminal output primitives
//!
//! Provides consistent, elegant output formatting for Anna Assistant.
//! Uses pastel colors and Unicode box drawing.

/// Strip ANSI escape codes from a string to get visible length
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;

    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// ANSI color codes - pastel palette
pub struct Colors;

impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BLUE: &'static str = "\x1b[38;5;117m";      // Pastel blue
    pub const GREEN: &'static str = "\x1b[38;5;120m";     // Pastel green
    pub const YELLOW: &'static str = "\x1b[38;5;228m";    // Pastel yellow
    pub const RED: &'static str = "\x1b[38;5;210m";       // Pastel red
    pub const GRAY: &'static str = "\x1b[38;5;250m";      // Light gray
    pub const CYAN: &'static str = "\x1b[38;5;159m";      // Pastel cyan
    pub const BOLD: &'static str = "\x1b[1m";
}

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

    pub fn color(&self) -> &'static str {
        match self {
            Level::Info => Colors::CYAN,
            Level::Success => Colors::GREEN,
            Level::Warning => Colors::YELLOW,
            Level::Error => Colors::RED,
        }
    }
}

/// Format a header
pub fn header(text: &str) -> String {
    let text_len = strip_ansi(text).len();
    let total_width = text_len + 4; // 2 spaces padding on each side
    let border = "─".repeat(total_width);
    let padding = " ".repeat(total_width - text_len - 2);

    format!(
        "{}{}╭{}╮\n│  {}{}  │\n╰{}╯{}",
        Colors::BOLD,
        Colors::BLUE,
        border,
        text,
        padding,
        border,
        Colors::RESET
    )
}

/// Format a section title
pub fn section(text: &str) -> String {
    format!(
        "{}{}{} {}{}",
        Colors::BOLD,
        Colors::CYAN,
        "→",
        text,
        Colors::RESET
    )
}

/// Format a status message
pub fn status(level: Level, message: &str) -> String {
    format!(
        "{}{} {}{}",
        level.color(),
        level.symbol(),
        message,
        Colors::RESET
    )
}

/// Format a key-value pair
pub fn kv(key: &str, value: &str) -> String {
    format!(
        "{}{}:{} {}",
        Colors::GRAY,
        key,
        Colors::RESET,
        value
    )
}

/// Format a box around text
pub fn boxed(lines: &[&str]) -> String {
    // Calculate max visible length (without ANSI codes)
    let max_len = lines.iter()
        .map(|l| strip_ansi(l).len())
        .max()
        .unwrap_or(0);

    let top = format!("{}╭{}╮{}", Colors::BLUE, "─".repeat(max_len + 2), Colors::RESET);
    let bottom = format!("{}╰{}╯{}", Colors::BLUE, "─".repeat(max_len + 2), Colors::RESET);

    let mut result = vec![top];
    for line in lines {
        let visible_len = strip_ansi(line).len();
        let padding = " ".repeat(max_len - visible_len);
        result.push(format!("{}│{} {}{} │{}", Colors::BLUE, Colors::RESET, line, padding, Colors::RESET));
    }
    result.push(bottom);

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status() {
        let msg = status(Level::Success, "Test passed");
        assert!(msg.contains("✓"));
    }
}
