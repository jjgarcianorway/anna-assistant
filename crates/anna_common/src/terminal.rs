//! Terminal Utilities v7.39.0 - Adaptive Rendering
//!
//! Provides terminal-size aware rendering:
//! - Compact mode for small terminals (< 24 rows or < 60 cols)
//! - Standard mode for normal terminals
//! - Wide mode for large terminals (> 120 cols)
//!
//! Also provides table rendering helpers for wide terminals.

use terminal_size::{terminal_size, Width, Height};

/// Minimum terminal width to function
pub const MIN_WIDTH: u16 = 40;

/// Width threshold for wide mode
pub const WIDE_WIDTH_THRESHOLD: u16 = 120;

/// Height threshold for compact mode
pub const COMPACT_HEIGHT_THRESHOLD: u16 = 24;

/// Width threshold for compact mode
pub const COMPACT_WIDTH_THRESHOLD: u16 = 60;

/// Display mode based on terminal size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Small terminal - show minimal info
    Compact,
    /// Normal terminal - standard sections
    Standard,
    /// Large terminal - extra columns and tables
    Wide,
}

impl DisplayMode {
    /// Detect display mode from current terminal
    pub fn detect() -> Self {
        let (width, height) = get_terminal_size();

        // Compact if terminal is small
        if height < COMPACT_HEIGHT_THRESHOLD || width < COMPACT_WIDTH_THRESHOLD {
            return DisplayMode::Compact;
        }

        // Wide if terminal is large
        if width >= WIDE_WIDTH_THRESHOLD {
            return DisplayMode::Wide;
        }

        DisplayMode::Standard
    }

    /// Get available width for content (accounting for margins)
    pub fn content_width(&self) -> u16 {
        let (width, _) = get_terminal_size();
        width.saturating_sub(4) // 2 char margin on each side
    }

    /// Should show this section in current mode?
    pub fn should_show_section(&self, section: &str, is_essential: bool) -> bool {
        match self {
            DisplayMode::Compact => {
                // In compact mode, only show essential sections
                is_essential || matches!(
                    section.to_uppercase().as_str(),
                    "[VERSION]" | "[DAEMON]" | "[HEALTH]" | "[ALERTS]" | "[UPDATES]"
                )
            }
            DisplayMode::Standard | DisplayMode::Wide => true,
        }
    }

    /// Maximum items to show in a list for this mode
    pub fn max_list_items(&self) -> usize {
        match self {
            DisplayMode::Compact => 3,
            DisplayMode::Standard => 5,
            DisplayMode::Wide => 10,
        }
    }
}

/// Get current terminal size with fallback
pub fn get_terminal_size() -> (u16, u16) {
    if let Some((Width(w), Height(h))) = terminal_size() {
        (w.max(MIN_WIDTH), h.max(10))
    } else {
        // Fallback for non-TTY (e.g., pipe)
        (80, 24)
    }
}

/// Truncate text to fit width, adding ellipsis if needed
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        return s.to_string();
    }

    if max_width <= 3 {
        return "...".to_string();
    }

    format!("{}...", &s[..max_width - 3])
}

/// Wrap text to width, preserving indent
pub fn wrap_text(s: &str, width: usize, indent: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let indent_str = " ".repeat(indent);
    let available = width.saturating_sub(indent);

    if available < 10 {
        // Too narrow, just return truncated
        lines.push(truncate(s, width));
        return lines;
    }

    let mut current_line = String::new();

    for word in s.split_whitespace() {
        if current_line.is_empty() {
            if word.len() > available {
                // Word itself is too long, truncate
                lines.push(truncate(word, available));
            } else {
                current_line = word.to_string();
            }
        } else if current_line.len() + 1 + word.len() <= available {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Start new line
            if lines.is_empty() {
                lines.push(current_line);
            } else {
                lines.push(format!("{}{}", indent_str, current_line));
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        if lines.is_empty() {
            lines.push(current_line);
        } else {
            lines.push(format!("{}{}", indent_str, current_line));
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Simple table builder for wide mode
pub struct SimpleTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    col_widths: Vec<usize>,
}

impl SimpleTable {
    /// Create a new table with headers
    pub fn new(headers: Vec<&str>) -> Self {
        let headers: Vec<String> = headers.into_iter().map(|s| s.to_string()).collect();
        let col_widths = headers.iter().map(|h| h.len()).collect();

        Self {
            headers,
            rows: Vec::new(),
            col_widths,
        }
    }

    /// Add a row to the table
    pub fn add_row(&mut self, cells: Vec<&str>) {
        let row: Vec<String> = cells.into_iter().map(|s| s.to_string()).collect();

        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.col_widths.len() {
                self.col_widths[i] = self.col_widths[i].max(cell.len());
            }
        }

        self.rows.push(row);
    }

    /// Render the table to strings
    pub fn render(&self, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();

        // Calculate if we need to truncate columns
        let total_width: usize = self.col_widths.iter().sum::<usize>()
            + (self.col_widths.len() - 1) * 3; // " | " separators

        let col_widths = if total_width > max_width && max_width > 20 {
            // Need to shrink columns proportionally
            let shrink_factor = max_width as f64 / total_width as f64;
            self.col_widths
                .iter()
                .map(|w| ((*w as f64 * shrink_factor) as usize).max(5))
                .collect()
        } else {
            self.col_widths.clone()
        };

        // Header line
        let header_line: String = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:width$}", truncate(h, col_widths[i]), width = col_widths[i]))
            .collect::<Vec<_>>()
            .join(" | ");
        lines.push(format!("  {}", header_line));

        // Separator
        let sep: String = col_widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>().join("-+-");
        lines.push(format!("  {}", sep));

        // Data rows
        for row in &self.rows {
            let row_line: String = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let w = col_widths.get(i).copied().unwrap_or(10);
                    format!("{:width$}", truncate(cell, w), width = w)
                })
                .collect::<Vec<_>>()
                .join(" | ");
            lines.push(format!("  {}", row_line));
        }

        lines
    }
}

/// Format a summary line with "(and N more)" if truncated
pub fn format_with_overflow(items: &[String], max_items: usize) -> String {
    if items.is_empty() {
        return "none".to_string();
    }

    if items.len() <= max_items {
        return items.join(", ");
    }

    let shown: Vec<_> = items.iter().take(max_items).cloned().collect();
    let remaining = items.len() - max_items;
    format!("{} (and {} more)", shown.join(", "), remaining)
}

/// Format a compact one-line summary
pub fn format_compact_line(label: &str, value: &str, max_width: usize) -> String {
    let label_len = label.len();
    let available = max_width.saturating_sub(label_len + 2); // ": "

    format!("{}: {}", label, truncate(value, available))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 3), "hi"); // Fits, no truncation needed
        assert_eq!(truncate("hello", 3), "..."); // Too long, but only 3 chars allowed
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a long line that needs wrapping to fit";
        let wrapped = wrap_text(text, 20, 2);
        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_format_with_overflow() {
        let items: Vec<String> = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
        assert_eq!(format_with_overflow(&items, 3), "a, b, c (and 2 more)");
        assert_eq!(format_with_overflow(&items, 10), "a, b, c, d, e");
    }

    #[test]
    fn test_simple_table() {
        let mut table = SimpleTable::new(vec!["ID", "Severity", "Summary"]);
        table.add_row(vec!["1", "warning", "Disk usage high"]);
        table.add_row(vec!["2", "critical", "Service failed"]);

        let lines = table.render(80);
        assert!(lines.len() >= 4); // Header + separator + 2 rows
    }
}
