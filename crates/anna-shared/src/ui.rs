//! Terminal UI helpers for consistent output styling.

use std::io::{self, Write};

/// ANSI color codes using true color (24-bit)
pub mod colors {
    pub const HEADER: &str = "\x1b[38;2;255;210;120m";
    pub const OK: &str = "\x1b[38;2;120;255;120m";
    pub const ERR: &str = "\x1b[38;2;255;100;100m";
    pub const WARN: &str = "\x1b[38;2;255;200;100m";
    pub const DIM: &str = "\x1b[38;2;140;140;140m";
    pub const CYAN: &str = "\x1b[38;2;100;200;255m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// Unicode symbols
pub mod symbols {
    pub const OK: &str = "✓";
    pub const ERR: &str = "✗";
    pub const ARROW: &str = "›";
    pub const SPINNER: [&str; 4] = ["⠋", "⠙", "⠹", "⠸"];
    pub const PROGRESS_FULL: &str = "█";
    pub const PROGRESS_EMPTY: &str = "░";
}

/// Horizontal rule
pub const HR: &str =
    "──────────────────────────────────────────────────────────────────────────────";

/// Print a styled header with version
pub fn print_header(name: &str, version: &str) {
    println!();
    println!("{}{} v{}{}", colors::HEADER, name, version, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Print a footer with horizontal rule
pub fn print_footer() {
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

/// Print a section header like [section] description
pub fn print_section(section: &str, description: &str) {
    println!(
        "{}[{}{}{}]{} {}",
        colors::DIM,
        colors::RESET,
        section,
        colors::DIM,
        colors::RESET,
        description
    );
}

/// Print an OK line with checkmark
pub fn print_ok(message: &str) {
    println!(
        "  {}{}{} {}",
        colors::OK,
        symbols::OK,
        colors::RESET,
        message
    );
}

/// Print an error line with X
pub fn print_err(message: &str) {
    println!(
        "  {}{}{} {}",
        colors::ERR,
        symbols::ERR,
        colors::RESET,
        message
    );
}

/// Print a key-value pair with alignment
pub fn print_kv(key: &str, value: &str, key_width: usize) {
    println!("  {:width$} {}", key, value, width = key_width);
}

/// Print a key-value pair with colored value
pub fn print_kv_status(key: &str, value: &str, status_color: &str, key_width: usize) {
    println!(
        "  {:width$} {}{}{}",
        key,
        status_color,
        value,
        colors::RESET,
        width = key_width
    );
}

/// Format a progress bar
pub fn progress_bar(progress: f32, width: usize) -> String {
    let filled = (progress * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}]",
        symbols::PROGRESS_FULL.repeat(filled),
        symbols::PROGRESS_EMPTY.repeat(empty)
    )
}

/// Format bytes as human readable
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human readable
pub fn format_duration(seconds: u64) -> String {
    if seconds >= 3600 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{:02}:{:02}:{:02}", hours, mins, seconds % 60)
    } else if seconds >= 60 {
        let mins = seconds / 60;
        format!("{:02}:{:02}", mins, seconds % 60)
    } else {
        format!("00:00:{:02}", seconds)
    }
}

/// Print without newline and flush
pub fn print_inline(message: &str) {
    print!("{}", message);
    io::stdout().flush().ok();
}

/// Clear current line
pub fn clear_line() {
    print!("\r\x1b[K");
    io::stdout().flush().ok();
}

/// Move cursor up n lines
pub fn cursor_up(n: usize) {
    print!("\x1b[{}A", n);
    io::stdout().flush().ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0.5, 10), "[█████░░░░░]");
        assert_eq!(progress_bar(1.0, 10), "[██████████]");
        assert_eq!(progress_bar(0.0, 10), "[░░░░░░░░░░]");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(5), "00:00:05");
        assert_eq!(format_duration(65), "01:05");
        assert_eq!(format_duration(3665), "01:01:05");
    }
}
