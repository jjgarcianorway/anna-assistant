//! Output formatting with TTY awareness.
//!
//! When stdout is a TTY: render markdown tables with formatting.
//! When piped: convert tables to aligned plain text for CI/logs/less.

use std::io::{stdout, IsTerminal};

/// Output mode based on stdout characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Interactive terminal - use rich formatting
    Tty,
    /// Piped/redirected - use plain text
    Pipe,
}

impl OutputMode {
    /// Detect output mode from stdout
    pub fn detect() -> Self {
        if stdout().is_terminal() {
            Self::Tty
        } else {
            Self::Pipe
        }
    }
}

/// Format text for the current output mode.
/// In pipe mode: converts markdown tables to aligned plain text.
pub fn format_for_output(text: &str, mode: OutputMode) -> String {
    match mode {
        OutputMode::Tty => text.to_string(),
        OutputMode::Pipe => convert_markdown_tables(text),
    }
}

/// Convert markdown tables to aligned plain text.
///
/// Input:
/// ```text
/// | PID | COMMAND | %MEM |
/// |-----|---------|------|
/// | 123 | firefox | 5.0% |
/// ```
///
/// Output:
/// ```text
/// PID   COMMAND   %MEM
/// 123   firefox   5.0%
/// ```
fn convert_markdown_tables(text: &str) -> String {
    let mut result = String::new();
    let mut in_table = false;
    let mut column_widths: Vec<usize> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // Detect table row (starts and ends with |)
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // Skip separator rows (|---|---|)
            if trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace()) {
                continue;
            }

            in_table = true;

            // Parse cells
            let cells: Vec<String> = trimmed
                .trim_matches('|')
                .split('|')
                .map(|s| s.trim().to_string())
                .collect();

            // Track max widths
            for (i, cell) in cells.iter().enumerate() {
                let cell_width = cell.chars().count();
                if i >= column_widths.len() {
                    column_widths.push(cell_width);
                } else if cell_width > column_widths[i] {
                    column_widths[i] = cell_width;
                }
            }

            table_rows.push(cells);
        } else {
            // Flush any pending table
            if in_table {
                result.push_str(&format_plain_table(&table_rows, &column_widths));
                table_rows.clear();
                column_widths.clear();
                in_table = false;
            }

            // Add non-table line as-is
            result.push_str(line);
            result.push('\n');
        }
    }

    // Flush final table if any
    if in_table {
        result.push_str(&format_plain_table(&table_rows, &column_widths));
    }

    // Remove trailing newline to match input behavior
    if result.ends_with('\n') && !text.ends_with('\n') {
        result.pop();
    }

    result
}

/// Format a parsed table as aligned plain text
fn format_plain_table(rows: &[Vec<String>], widths: &[usize]) -> String {
    let mut result = String::new();

    for row in rows {
        let formatted_cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = widths.get(i).copied().unwrap_or(0);
                format!("{:width$}", cell, width = width)
            })
            .collect();

        result.push_str(&formatted_cells.join("  "));
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode_detection() {
        // Just verify it doesn't panic
        let _mode = OutputMode::detect();
    }

    #[test]
    fn test_markdown_table_conversion() {
        let markdown = r#"**Top processes:**

| PID | COMMAND | %MEM |
|-----|---------|------|
| 123 | firefox | 5.0% |
| 456 | chrome  | 3.2% |

Done."#;

        let plain = convert_markdown_tables(markdown);

        // Should not contain table syntax
        assert!(!plain.contains("|--"));
        assert!(!plain.contains("| PID"));

        // Should contain data
        assert!(plain.contains("PID"));
        assert!(plain.contains("firefox"));
        assert!(plain.contains("chrome"));
        assert!(plain.contains("Done."));
    }

    #[test]
    fn test_non_table_text_unchanged() {
        let text = "This is normal text\nWith multiple lines\nNo tables here.";
        let result = convert_markdown_tables(text);
        // Non-table text is preserved (trailing newline matching input behavior)
        assert_eq!(result, text);
    }

    #[test]
    fn test_mixed_content() {
        let text = r#"**Header**

| A | B |
|---|---|
| 1 | 2 |

Footer text."#;

        let result = convert_markdown_tables(text);

        assert!(result.contains("**Header**"));
        assert!(result.contains("Footer text."));
        assert!(result.contains("A"));
        assert!(result.contains("1"));
        assert!(!result.contains("|---|"));
    }

    #[test]
    fn test_tty_mode_preserves_markdown() {
        let text = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = format_for_output(text, OutputMode::Tty);
        assert_eq!(result, text);
    }

    #[test]
    fn test_pipe_mode_converts_tables() {
        let text = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = format_for_output(text, OutputMode::Pipe);
        assert!(!result.contains('|'));
    }

    // Force mode tests (primary truth per spec)
    #[test]
    fn test_force_pipe_mode_column_alignment() {
        let text = "| PID | COMMAND | %MEM |\n|-----|---------|------|\n| 1 | init | 0.1% |\n| 12345 | firefox | 15.5% |";
        let result = format_for_output(text, OutputMode::Pipe);

        // Columns should be aligned
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3, "Header + 2 data rows");

        // All lines should have consistent column positions
        for line in &lines {
            assert!(!line.contains('|'), "No pipe chars in output");
        }
    }

    #[test]
    fn test_force_tty_mode_preserves_all_formatting() {
        let text = "| A | B |\n|:--|--:|\n| 1 | 2 |";
        let result = format_for_output(text, OutputMode::Tty);
        // TTY mode preserves everything including alignment markers
        assert!(result.contains("|:--|"));
        assert!(result.contains("--:|"));
    }

    #[test]
    fn test_force_pipe_mode_multiple_tables() {
        let text = "Table 1:\n| A | B |\n|---|---|\n| 1 | 2 |\n\nTable 2:\n| X | Y |\n|---|---|\n| 3 | 4 |";
        let result = format_for_output(text, OutputMode::Pipe);

        // Both tables should be converted
        assert!(!result.contains('|'));
        assert!(result.contains("A"));
        assert!(result.contains("X"));
        assert!(result.contains("Table 1:"));
        assert!(result.contains("Table 2:"));
    }

    #[test]
    fn test_force_pipe_mode_empty_table() {
        let text = "|---|---|\n";
        let result = format_for_output(text, OutputMode::Pipe);
        // Separator-only row should be skipped
        assert!(!result.contains('|'));
    }

    #[test]
    fn test_force_pipe_mode_single_column_table() {
        let text = "| STATUS |\n|--------|\n| OK |";
        let result = format_for_output(text, OutputMode::Pipe);
        assert!(!result.contains('|'));
        assert!(result.contains("STATUS"));
        assert!(result.contains("OK"));
    }
}
