//! Terminal User Interface toolkit for Anna Assistant
//!
//! Provides consistent, retro DOS/Borland-style formatting across all Anna components.
//! All formatting respects NO_COLOR, CLICOLOR, and terminal capabilities.

use crate::messaging::TERM_CAPS;
use std::fmt::Write as FmtWrite;

/// Terminal capability structure with emoji support flag
#[derive(Debug, Clone)]
pub struct TermCaps {
    pub color: bool,
    pub emoji: bool,
    pub utf8: bool,
    pub width: usize,
}

impl TermCaps {
    /// Detect current terminal capabilities
    pub fn detect() -> Self {
        let caps = &*TERM_CAPS;
        let config = crate::config::default_config();

        Self {
            color: config.colors && caps.supports_color,
            emoji: config.emojis && caps.supports_unicode,
            utf8: caps.supports_unicode,
            width: caps.width.max(60).min(120), // Clamp to [60, 120]
        }
    }

    /// Create caps with all features enabled (for testing)
    pub fn full() -> Self {
        Self {
            color: true,
            emoji: true,
            utf8: true,
            width: 80,
        }
    }

    /// Create caps with no features (ASCII only)
    pub fn none() -> Self {
        Self {
            color: false,
            emoji: false,
            utf8: false,
            width: 80,
        }
    }
}

/// Message level for status lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Ok,
    Warn,
    Err,
}

/// ANSI color codes
mod ansi {
    pub const CYAN: &str = "\x1b[38;5;87m"; // primary
    pub const GREEN: &str = "\x1b[38;5;120m"; // ok
    pub const YELLOW: &str = "\x1b[38;5;228m"; // warn
    pub const RED: &str = "\x1b[38;5;210m"; // err
    pub const MAGENTA: &str = "\x1b[38;5;213m"; // accent
    pub const GRAY_250: &str = "\x1b[38;5;250m"; // fg default
    pub const GRAY_240: &str = "\x1b[38;5;240m"; // dim
    pub const RESET: &str = "\x1b[0m";
}

/// Symbols with ASCII fallbacks
struct Symbols;

impl Symbols {
    fn ok(emoji: bool) -> &'static str {
        if emoji {
            "✓"
        } else {
            "OK"
        }
    }

    fn warn(emoji: bool) -> &'static str {
        if emoji {
            "⚠"
        } else {
            "!"
        }
    }

    fn err(emoji: bool) -> &'static str {
        if emoji {
            "✗"
        } else {
            "X"
        }
    }

    fn bullet(emoji: bool) -> &'static str {
        if emoji {
            "•"
        } else {
            "-"
        }
    }

    fn play(emoji: bool) -> &'static str {
        if emoji {
            "▶"
        } else {
            ">"
        }
    }
}

/// Color a string with the specified ANSI code
fn colorize(caps: &TermCaps, code: &str, text: &str) -> String {
    if caps.color {
        format!("{}{}{}", code, text, ansi::RESET)
    } else {
        text.to_string()
    }
}

/// Dim text (gray 240)
pub fn dim(caps: &TermCaps, text: &str) -> String {
    colorize(caps, ansi::GRAY_240, text)
}

/// OK text (green)
pub fn ok(caps: &TermCaps, text: &str) -> String {
    colorize(caps, ansi::GREEN, text)
}

/// Warning text (yellow)
pub fn warn(caps: &TermCaps, text: &str) -> String {
    colorize(caps, ansi::YELLOW, text)
}

/// Error text (red)
pub fn err(caps: &TermCaps, text: &str) -> String {
    colorize(caps, ansi::RED, text)
}

/// Primary text (cyan)
pub fn primary(caps: &TermCaps, text: &str) -> String {
    colorize(caps, ansi::CYAN, text)
}

/// Accent text (magenta)
pub fn accent(caps: &TermCaps, text: &str) -> String {
    colorize(caps, ansi::MAGENTA, text)
}

/// Create a header box (top-level title)
///
/// ```text
/// ╭─ Anna Hardware Profile ──────────────╮
/// ```
pub fn header(caps: &TermCaps, title: &str) -> String {
    let (tl, h, tr) = if caps.utf8 {
        ("╭─ ", "─", " ╮")
    } else {
        ("+- ", "-", " +")
    };

    let title_len = title.len();
    let padding_total = caps.width.saturating_sub(title_len + 6); // 6 for borders and spaces
    let padding = h.repeat(padding_total);

    let line = format!("{}{}{}{}", tl, title, padding, tr);

    if caps.color {
        primary(caps, &line)
    } else {
        line
    }
}

/// Create a section divider
///
/// ```text
/// ────────────────────────────────────────
/// ```
pub fn section(caps: &TermCaps, title: &str) -> String {
    let h = if caps.utf8 { "─" } else { "-" };

    if title.is_empty() {
        // Just a line
        let line = h.repeat(caps.width.min(40));
        dim(caps, &line)
    } else {
        // Title with separator
        let title_formatted = if caps.color {
            primary(caps, title)
        } else {
            title.to_string()
        };
        title_formatted
    }
}

/// Create a key-value line with aligned column
///
/// ```text
/// CPU:       AMD Ryzen 9 5900X 16 cores
/// Memory:    32.0 GB
/// ```
pub fn kv(caps: &TermCaps, key: &str, val: &str) -> String {
    let key_width = 12; // Fixed key column width
    let key_formatted = if caps.color {
        format!(
            "{}{:width$}{}",
            ansi::GRAY_250,
            key,
            ansi::RESET,
            width = key_width
        )
    } else {
        format!("{:width$}", key, width = key_width)
    };

    format!("{}{}", key_formatted, val)
}

/// Create a status line with symbol
///
/// ```text
/// ✓ Installation complete
/// ⚠ dmidecode not available
/// ✗ Failed to start daemon
/// ```
pub fn status(caps: &TermCaps, level: Level, msg: &str) -> String {
    let (symbol, color_fn): (&str, fn(&TermCaps, &str) -> String) = match level {
        Level::Info => (Symbols::play(caps.emoji), primary as _),
        Level::Ok => (Symbols::ok(caps.emoji), ok as _),
        Level::Warn => (Symbols::warn(caps.emoji), warn as _),
        Level::Err => (Symbols::err(caps.emoji), err as _),
    };

    if caps.color {
        format!("{} {}", color_fn(caps, symbol), msg)
    } else {
        format!("{} {}", symbol, msg)
    }
}

/// Create a bullet point
///
/// ```text
/// • Item one
/// • Item two
/// ```
pub fn bullet(caps: &TermCaps, msg: &str) -> String {
    format!("  {} {}", Symbols::bullet(caps.emoji), msg)
}

/// Create a hint line (dimmed)
///
/// ```text
/// Try: sudo systemctl status annad
/// ```
pub fn hint(caps: &TermCaps, msg: &str) -> String {
    dim(caps, msg)
}

/// Create a code block (fenced)
///
/// ```text
/// $ annactl hw show --json
/// ```
pub fn code(caps: &TermCaps, cmd: &str) -> String {
    if caps.color {
        format!("{}$ {}{}", ansi::GRAY_240, ansi::RESET, cmd)
    } else {
        format!("$ {}", cmd)
    }
}

/// Create a progress indicator
///
/// ```text
/// [2/5] Installing binaries
/// ```
pub fn progress(caps: &TermCaps, label: &str, step: usize, total: usize) -> String {
    let progress_str = format!("[{}/{}]", step, total);
    if caps.color {
        format!("{} {}", dim(caps, &progress_str), label)
    } else {
        format!("{} {}", progress_str, label)
    }
}

/// Create a simple ASCII table
///
/// Columns are auto-sized and truncated with "…" if needed.
/// Numbers are right-aligned, text is left-aligned.
pub fn table(caps: &TermCaps, headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() || rows.is_empty() {
        return String::new();
    }

    let num_cols = headers.len();

    // Calculate column widths
    let mut col_widths = vec![0; num_cols];
    for (i, header) in headers.iter().enumerate() {
        col_widths[i] = header.len();
    }

    for row in rows {
        for (i, cell) in row.iter().take(num_cols).enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    // Apply max width constraint (share available width)
    let total_width: usize = col_widths.iter().sum();
    let available = caps.width.saturating_sub(num_cols * 3); // Account for column spacing
    if total_width > available {
        // Scale down proportionally
        let scale = available as f64 / total_width as f64;
        for width in &mut col_widths {
            *width = ((*width as f64 * scale) as usize).max(4); // Minimum 4 chars
        }
    }

    let mut output = String::new();

    // Header row
    let mut header_line = String::new();
    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            header_line.push_str("  ");
        }
        let truncated = truncate(header, col_widths[i], caps.utf8);
        header_line.push_str(&format!("{:width$}", truncated, width = col_widths[i]));
    }

    if caps.color {
        writeln!(output, "{}", primary(caps, &header_line)).ok();
    } else {
        writeln!(output, "{}", header_line).ok();
    }

    // Separator
    let sep = if caps.utf8 { "─" } else { "-" };
    let sep_line: String = col_widths
        .iter()
        .enumerate()
        .map(|(i, &w)| {
            if i > 0 {
                format!("  {}", sep.repeat(w))
            } else {
                sep.repeat(w)
            }
        })
        .collect();

    writeln!(output, "{}", dim(caps, &sep_line)).ok();

    // Data rows
    for row in rows {
        let mut row_line = String::new();
        for (i, cell) in row.iter().take(num_cols).enumerate() {
            if i > 0 {
                row_line.push_str("  ");
            }

            let truncated = truncate(cell, col_widths[i], caps.utf8);

            // Right-align if looks like a number, left-align otherwise
            if is_numeric(cell) {
                row_line.push_str(&format!("{:>width$}", truncated, width = col_widths[i]));
            } else {
                row_line.push_str(&format!("{:width$}", truncated, width = col_widths[i]));
            }
        }
        writeln!(output, "{}", row_line).ok();
    }

    output
}

/// Truncate a string to fit within max_len, adding ellipsis if needed
fn truncate(s: &str, max_len: usize, use_unicode: bool) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    let ellipsis = if use_unicode { "…" } else { "..." };
    let ellipsis_len = if use_unicode { 1 } else { 3 };

    if max_len <= ellipsis_len {
        return ellipsis[..max_len.min(ellipsis.len())].to_string();
    }

    let truncate_at = max_len - ellipsis_len;
    format!("{}{}", &s[..truncate_at], ellipsis)
}

/// Check if a string looks like a number
fn is_numeric(s: &str) -> bool {
    s.trim()
        .chars()
        .all(|c| c.is_numeric() || c == '.' || c == ',' || c == '%' || c == '-')
}

/// Create a progress bar (text-based)
///
/// ```text
/// ████████████░░░░░░░░ 60%
/// ```
pub fn progress_bar(caps: &TermCaps, pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0 * width as f32) as usize).min(width);
    let empty = width.saturating_sub(filled);

    let (fill_char, empty_char) = if caps.utf8 {
        ('█', '░')
    } else {
        ('#', '-')
    };

    let bar = format!(
        "{}{}",
        fill_char.to_string().repeat(filled),
        empty_char.to_string().repeat(empty)
    );

    if caps.color {
        if pct >= 90.0 {
            ok(caps, &bar)
        } else if pct >= 70.0 {
            primary(caps, &bar)
        } else if pct >= 50.0 {
            warn(caps, &bar)
        } else {
            err(caps, &bar)
        }
    } else {
        bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caps_detection() {
        let caps = TermCaps::detect();
        assert!(caps.width >= 60 && caps.width <= 120);
    }

    #[test]
    fn test_header_utf8() {
        let caps = TermCaps::full();
        let h = header(&caps, "Test");
        assert!(h.contains("╭─"));
        assert!(h.contains("Test"));
    }

    #[test]
    fn test_header_ascii() {
        let caps = TermCaps::none();
        let h = header(&caps, "Test");
        assert!(h.contains("+-"));
        assert!(h.contains("Test"));
        assert!(!h.contains('\x1b')); // No ANSI codes
    }

    #[test]
    fn test_status_symbols() {
        let caps = TermCaps::full();
        assert!(status(&caps, Level::Ok, "test").contains("✓"));

        let caps = TermCaps::none();
        assert!(status(&caps, Level::Ok, "test").contains("OK"));
    }

    #[test]
    fn test_kv_alignment() {
        let caps = TermCaps::none();
        let line1 = kv(&caps, "CPU:", "AMD Ryzen");
        let line2 = kv(&caps, "Memory:", "32 GB");
        // Both lines should have same format/structure
        // The key takes 12 chars total
        assert!(line1.starts_with("CPU:"));
        assert!(line2.starts_with("Memory:"));
        assert!(line1.len() > 12);
        assert!(line2.len() > 12);
    }

    #[test]
    fn test_table_basic() {
        let caps = TermCaps::none();
        let headers = vec!["Name", "Value"];
        let rows = vec![
            vec!["CPU".to_string(), "100%".to_string()],
            vec!["Memory".to_string(), "50%".to_string()],
        ];

        let table_str = table(&caps, &headers, &rows);
        assert!(table_str.contains("Name"));
        assert!(table_str.contains("CPU"));
        assert!(table_str.contains("100%"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10, true), "hello");
        assert_eq!(truncate("hello world", 8, true), "hello w…");
        assert_eq!(truncate("hello world", 8, false), "hello...");
    }

    #[test]
    fn test_progress_bar() {
        let caps = TermCaps {
            color: false,
            emoji: false,
            utf8: true,
            width: 80,
        };

        let bar = progress_bar(&caps, 50.0, 20);
        // UTF-8 chars are multiple bytes, so byte length != visual length
        // Just check it contains the right characters
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));

        // Count visual characters
        let visual_len = bar.chars().count();
        assert_eq!(visual_len, 20);
    }

    #[test]
    fn test_no_color_in_ascii_mode() {
        let caps = TermCaps::none();
        let colored = ok(&caps, "test");
        assert!(!colored.contains('\x1b'));
    }
}
