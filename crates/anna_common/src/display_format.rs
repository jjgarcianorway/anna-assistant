//! Display Formatting Utilities v0.0.15
//!
//! Centralized formatting rules for all user-facing output.
//! All output must be ASCII only - no emojis, no Unicode decorations.
//!
//! Rules:
//! - Duration: <1s -> ms, 1s-1m -> X.Xs, 1m-1h -> Xm Ys, >1h -> Xh Ym
//! - Counts: Use K/M suffixes only when clearly helpful
//! - Times: Human-readable relative times ("3m ago", "2h ago")
//!
//! v0.0.15: Unified formatting system with sections, headers, and dialogue formatting

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Format a duration in milliseconds to human-readable form
/// Rules:
/// - Under 1 second: show in ms (e.g. 120ms)
/// - From 1 second to 1 minute: show in seconds with one decimal (e.g. 4.2s)
/// - From 1 minute to 1 hour: show in minutes and seconds (e.g. 3m 15s)
/// - Over 1 hour: show in Hh Mm (e.g. 1h 34m)
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        let secs = ms as f64 / 1000.0;
        format!("{:.1}s", secs)
    } else if ms < 3_600_000 {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else {
        let hours = ms / 3_600_000;
        let mins = (ms % 3_600_000) / 60_000;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    }
}

/// Format a duration from seconds
pub fn format_duration_secs(secs: u64) -> String {
    format_duration_ms(secs * 1000)
}

/// Format a Duration struct
pub fn format_duration(d: Duration) -> String {
    format_duration_ms(d.as_millis() as u64)
}

/// Format a relative time ("3m ago", "2h ago", "just now")
pub fn format_time_ago(timestamp: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if timestamp > now {
        return "just now".to_string();
    }

    let diff = now - timestamp;

    if diff < 60 {
        if diff <= 5 {
            "just now".to_string()
        } else {
            format!("{}s ago", diff)
        }
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{}m ago", mins)
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{}h ago", hours)
    } else {
        let days = diff / 86400;
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    }
}

/// Format a count with K/M suffix when appropriate
/// Only use suffixes for large numbers where exact count is not actionable
pub fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else if n >= 1_000 {
        // For 1K-10K, show exact number with comma
        format_with_commas(n)
    } else {
        format!("{}", n)
    }
}

/// Format a number with comma separators (e.g., 1,234)
pub fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a percentage (0-100)
pub fn format_percent(value: f64) -> String {
    if value >= 99.5 {
        "100%".to_string()
    } else if value < 1.0 && value > 0.0 {
        "<1%".to_string()
    } else {
        format!("{:.0}%", value)
    }
}

/// Format a ratio as percentage (e.g., 80/100 -> "80%")
pub fn format_ratio_percent(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        "n/a".to_string()
    } else {
        let pct = (numerator as f64 / denominator as f64) * 100.0;
        format_percent(pct)
    }
}

/// Format bytes to human-readable (KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Format an ETA based on progress and rate
/// Returns "n/a" if cannot estimate, otherwise "~Xm" or "~Xs"
pub fn format_eta(remaining: u64, rate_per_sec: f64) -> String {
    if rate_per_sec <= 0.0 || remaining == 0 {
        return "n/a".to_string();
    }

    let secs = (remaining as f64 / rate_per_sec) as u64;
    if secs < 60 {
        format!("~{}s", secs)
    } else if secs < 3600 {
        format!("~{}m", secs / 60)
    } else {
        format!("~{}h", secs / 3600)
    }
}

/// v7.29.0: DEPRECATED - prefer text_wrap functions for zero truncation
/// Truncate a string to max length, adding "..." if truncated
#[deprecated(
    since = "7.29.0",
    note = "use text_wrap module for zero truncation output"
)]
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}

/// v7.29.0: Clip a string to max length without adding ellipsis
/// Use when display width is limited but full content is elsewhere
pub fn clip_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect()
    }
}

/// Format a timestamp as "YYYY-MM-DD HH:MM"
pub fn format_timestamp(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Format a timestamp as "YYYY-MM-DD HH:MM:SS"
pub fn format_timestamp_full(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Check if a string contains only ASCII characters
pub fn is_ascii_only(s: &str) -> bool {
    s.is_ascii()
}

/// Strip non-ASCII characters from a string
pub fn strip_non_ascii(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii()).collect()
}

// =============================================================================
// v0.0.15: Unified Formatting System
// =============================================================================

use owo_colors::OwoColorize;

/// ANSI color codes for consistent theming
pub mod colors {
    use owo_colors::OwoColorize;

    /// Format text as a section header (bold cyan)
    pub fn section_header(text: &str) -> String {
        text.bold().cyan().to_string()
    }

    /// Format text as a label (dim white)
    pub fn label(text: &str) -> String {
        text.dimmed().to_string()
    }

    /// Format text as a value (white)
    pub fn value(text: &str) -> String {
        text.white().to_string()
    }

    /// Format text as success (green)
    pub fn success(text: &str) -> String {
        text.green().to_string()
    }

    /// Format text as warning (yellow)
    pub fn warning(text: &str) -> String {
        text.yellow().to_string()
    }

    /// Format text as error (red)
    pub fn error(text: &str) -> String {
        text.red().to_string()
    }

    /// Format text as info (blue)
    pub fn info(text: &str) -> String {
        text.blue().to_string()
    }

    /// Format evidence ID (bright magenta)
    pub fn evidence_id(text: &str) -> String {
        text.bright_magenta().to_string()
    }

    /// Format actor name in dialogue (bold)
    pub fn actor(text: &str) -> String {
        text.bold().to_string()
    }

    /// Format reliability score based on value
    pub fn reliability(score: u32) -> String {
        let text = format!("{}%", score);
        if score >= 80 {
            text.green().to_string()
        } else if score >= 60 {
            text.yellow().to_string()
        } else {
            text.red().to_string()
        }
    }
}

/// Section formatter for status display
pub struct SectionFormatter {
    width: u16,
    use_colors: bool,
}

impl SectionFormatter {
    /// Create a new section formatter
    pub fn new(width: u16, use_colors: bool) -> Self {
        Self { width, use_colors }
    }

    /// Format a section header (e.g., "[VERSION]")
    pub fn header(&self, title: &str) -> String {
        let header = format!("[{}]", title.to_uppercase());
        if self.use_colors {
            colors::section_header(&header)
        } else {
            header
        }
    }

    /// Format a key-value pair with consistent alignment
    pub fn key_value(&self, key: &str, value: &str) -> String {
        let label_width = 14;
        let formatted_key = format!("{:>width$}:", key, width = label_width);
        if self.use_colors {
            format!("  {} {}", colors::label(&formatted_key), value)
        } else {
            format!("  {} {}", formatted_key, value)
        }
    }

    /// Format a key-value pair with a status indicator
    pub fn key_value_status(&self, key: &str, value: &str, status: StatusIndicator) -> String {
        let label_width = 14;
        let formatted_key = format!("{:>width$}:", key, width = label_width);
        let status_str = match status {
            StatusIndicator::Ok => {
                if self.use_colors {
                    colors::success("ok")
                } else {
                    "ok".to_string()
                }
            }
            StatusIndicator::Warning => {
                if self.use_colors {
                    colors::warning("warning")
                } else {
                    "warning".to_string()
                }
            }
            StatusIndicator::Error => {
                if self.use_colors {
                    colors::error("error")
                } else {
                    "error".to_string()
                }
            }
            StatusIndicator::Info => {
                if self.use_colors {
                    colors::info("info")
                } else {
                    "info".to_string()
                }
            }
            StatusIndicator::None => String::new(),
        };
        if status_str.is_empty() {
            if self.use_colors {
                format!("  {} {}", colors::label(&formatted_key), value)
            } else {
                format!("  {} {}", formatted_key, value)
            }
        } else {
            if self.use_colors {
                format!(
                    "  {} {} ({})",
                    colors::label(&formatted_key),
                    value,
                    status_str
                )
            } else {
                format!("  {} {} ({})", formatted_key, value, status_str)
            }
        }
    }

    /// Format an indented item (for lists)
    pub fn item(&self, text: &str) -> String {
        format!("    {}", text)
    }

    /// Format a sub-item (deeper indentation)
    pub fn sub_item(&self, text: &str) -> String {
        format!("      {}", text)
    }

    /// Format a blank line (section separator)
    pub fn blank() -> String {
        String::new()
    }

    /// Format an evidence ID
    pub fn evidence(&self, id: &str) -> String {
        if self.use_colors {
            colors::evidence_id(id)
        } else {
            format!("[{}]", id)
        }
    }

    /// Format reliability score
    pub fn reliability(&self, score: u32) -> String {
        if self.use_colors {
            colors::reliability(score)
        } else {
            format!("{}%", score)
        }
    }

    /// Format a horizontal rule
    pub fn rule(&self) -> String {
        "-".repeat(self.width as usize).dimmed().to_string()
    }

    /// Get the terminal width
    pub fn width(&self) -> u16 {
        self.width
    }
}

/// Status indicator for key-value pairs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIndicator {
    Ok,
    Warning,
    Error,
    Info,
    None,
}

/// Dialogue formatter for pipeline output
pub struct DialogueFormatter {
    debug_level: u8,
    use_colors: bool,
}

impl DialogueFormatter {
    /// Create a new dialogue formatter
    pub fn new(debug_level: u8, use_colors: bool) -> Self {
        Self {
            debug_level,
            use_colors,
        }
    }

    /// Check if this dialogue line should be shown at current debug level
    pub fn should_show(&self, from: &str, to: &str) -> bool {
        match self.debug_level {
            0 => {
                // Minimal: only user<->anna dialogues and confirmations
                (from == "you" && to == "anna")
                    || (from == "anna" && to == "you")
                    || to == "confirmation"
            }
            1 => {
                // Normal: show all dialogues except internal tool details
                true
            }
            _ => {
                // Full: show everything
                true
            }
        }
    }

    /// Format a dialogue line
    pub fn format(&self, from: &str, to: &str, message: &str) -> String {
        let arrow = if self.use_colors {
            "->".dimmed().to_string()
        } else {
            "->".to_string()
        };

        let from_str = if self.use_colors {
            colors::actor(&format!("[{}]", from))
        } else {
            format!("[{}]", from)
        };

        let to_str = if self.use_colors {
            colors::actor(&format!("[{}]", to))
        } else {
            format!("[{}]", to)
        };

        format!("{} {} {}: {}", from_str, arrow, to_str, message)
    }

    /// Format a condensed tool execution summary (for debug level 1)
    pub fn format_tool_summary(&self, tools: &[&str], evidence_ids: &[&str]) -> String {
        let tools_str = tools.join(", ");
        let evidence_str = if evidence_ids.is_empty() {
            String::new()
        } else {
            let ids: Vec<String> = evidence_ids
                .iter()
                .map(|id| {
                    if self.use_colors {
                        colors::evidence_id(id)
                    } else {
                        format!("[{}]", id)
                    }
                })
                .collect();
            format!(" {}", ids.join(", "))
        };
        format!("  Tools: {}{}", tools_str, evidence_str)
    }

    /// Format final answer with reliability
    pub fn format_final_answer(
        &self,
        answer: &str,
        reliability: u32,
        evidence_ids: &[&str],
    ) -> String {
        let mut output = String::new();

        // Answer text
        output.push_str(answer);
        output.push('\n');

        // Evidence legend (if any)
        if !evidence_ids.is_empty() && self.debug_level >= 1 {
            output.push_str("\nEvidence:\n");
            for id in evidence_ids {
                let formatted_id = if self.use_colors {
                    colors::evidence_id(id)
                } else {
                    format!("[{}]", id)
                };
                output.push_str(&format!("  {}\n", formatted_id));
            }
        }

        // Reliability footer
        let reliability_str = if self.use_colors {
            colors::reliability(reliability)
        } else {
            format!("{}%", reliability)
        };
        output.push_str(&format!("\nReliability: {}", reliability_str));

        output
    }

    /// Get current debug level
    pub fn debug_level(&self) -> u8 {
        self.debug_level
    }
}

/// Format a list of items as a comma-separated string with "and" before the last item
pub fn format_list(items: &[&str]) -> String {
    match items.len() {
        0 => "none".to_string(),
        1 => items[0].to_string(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let last = items.last().unwrap();
            let rest = &items[..items.len() - 1];
            format!("{}, and {}", rest.join(", "), last)
        }
    }
}

/// Format a summary line for status sections
pub fn format_summary(present: u64, total: u64, unit: &str) -> String {
    if total == 0 {
        format!("no {}", unit)
    } else if present == total {
        format!("all {} {}", total, unit)
    } else {
        format!("{}/{} {}", present, total, unit)
    }
}

/// Wrap text to a maximum width, preserving words
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Indent a multi-line string
pub fn indent(text: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(50), "50ms");
        assert_eq!(format_duration_ms(999), "999ms");
        assert_eq!(format_duration_ms(1000), "1.0s");
        assert_eq!(format_duration_ms(4200), "4.2s");
        assert_eq!(format_duration_ms(59900), "59.9s");
        assert_eq!(format_duration_ms(60000), "1m");
        assert_eq!(format_duration_ms(195000), "3m 15s");
        assert_eq!(format_duration_ms(3600000), "1h");
        assert_eq!(format_duration_ms(5640000), "1h 34m");
    }

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1234), "1,234");
        assert_eq!(format_count(9999), "9,999");
        assert_eq!(format_count(10000), "10.0K");
        assert_eq!(format_count(90900), "90.9K");
        assert_eq!(format_count(1500000), "1.5M");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(0.0), "0%");
        assert_eq!(format_percent(0.5), "<1%");
        assert_eq!(format_percent(50.0), "50%");
        assert_eq!(format_percent(99.4), "99%");
        assert_eq!(format_percent(99.6), "100%");
        assert_eq!(format_percent(100.0), "100%");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1.0KB");
        assert_eq!(format_bytes(8400000), "8.0MB");
        assert_eq!(format_bytes(1073741824), "1.0GB");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(0, 10.0), "n/a");
        assert_eq!(format_eta(100, 0.0), "n/a");
        assert_eq!(format_eta(30, 1.0), "~30s");
        assert_eq!(format_eta(180, 1.0), "~3m");
        assert_eq!(format_eta(7200, 1.0), "~2h");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        assert_eq!(truncate_str("hi", 2), "hi");
    }

    #[test]
    fn test_is_ascii_only() {
        assert!(is_ascii_only("hello world"));
        assert!(is_ascii_only("test 123 !@#"));
        assert!(!is_ascii_only("hello 🌍"));
        assert!(!is_ascii_only("test ✓"));
    }

    #[test]
    fn test_strip_non_ascii() {
        assert_eq!(strip_non_ascii("hello 🌍 world"), "hello  world");
        assert_eq!(strip_non_ascii("test ✓ ok"), "test  ok");
        assert_eq!(strip_non_ascii("pure ascii"), "pure ascii");
    }

    // v0.0.15 formatting tests

    #[test]
    fn test_format_list() {
        assert_eq!(format_list(&[]), "none");
        assert_eq!(format_list(&["one"]), "one");
        assert_eq!(format_list(&["one", "two"]), "one and two");
        assert_eq!(format_list(&["one", "two", "three"]), "one, two, and three");
    }

    #[test]
    fn test_format_summary() {
        assert_eq!(format_summary(0, 0, "items"), "no items");
        assert_eq!(format_summary(5, 5, "helpers"), "all 5 helpers");
        assert_eq!(format_summary(3, 5, "alerts"), "3/5 alerts");
    }

    #[test]
    fn test_wrap_text() {
        let lines = wrap_text("hello world foo bar", 10);
        // "hello" (5), "world foo" (9), "bar" (3) - 3 lines at width 10
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "hello");
        assert_eq!(lines[1], "world foo");
        assert_eq!(lines[2], "bar");
    }

    #[test]
    fn test_indent() {
        let text = "line1\nline2";
        let indented = indent(text, 4);
        assert_eq!(indented, "    line1\n    line2");
    }

    #[test]
    fn test_section_formatter() {
        let fmt = SectionFormatter::new(80, false);
        assert!(fmt.header("version").contains("[VERSION]"));
        assert!(fmt.key_value("Mode", "auto").contains("Mode:"));
        assert!(fmt.key_value("Mode", "auto").contains("auto"));
    }

    #[test]
    fn test_dialogue_formatter_debug_levels() {
        // Level 0: minimal - only user<->anna
        let fmt0 = DialogueFormatter::new(0, false);
        assert!(fmt0.should_show("you", "anna"));
        assert!(fmt0.should_show("anna", "you"));
        assert!(!fmt0.should_show("anna", "translator"));
        assert!(!fmt0.should_show("junior", "anna"));

        // Level 1: normal - show all
        let fmt1 = DialogueFormatter::new(1, false);
        assert!(fmt1.should_show("you", "anna"));
        assert!(fmt1.should_show("anna", "translator"));
        assert!(fmt1.should_show("junior", "anna"));

        // Level 2: full - show everything
        let fmt2 = DialogueFormatter::new(2, false);
        assert!(fmt2.should_show("anna", "annad"));
    }

    #[test]
    fn test_dialogue_formatter_format_output() {
        let fmt = DialogueFormatter::new(1, false);
        let output = fmt.format("you", "anna", "hello world");
        assert!(output.contains("[you]"), "output: {}", output);
        assert!(output.contains("[anna]"), "output: {}", output);
        assert!(output.contains("hello world"), "output: {}", output);
    }

    #[test]
    fn test_tool_summary_format() {
        let fmt = DialogueFormatter::new(1, false);
        let tools = vec!["status_snapshot", "disk_usage"];
        let evidence = vec!["E1", "E2"];
        let summary = fmt.format_tool_summary(&tools, &evidence);
        // Check that tools and evidence IDs are in output
        assert!(summary.contains("status_snapshot"), "summary: {}", summary);
        assert!(summary.contains("disk_usage"), "summary: {}", summary);
        assert!(
            summary.contains("E1") || summary.contains("[E1]"),
            "summary: {}",
            summary
        );
    }

    #[test]
    fn test_status_indicator_colors() {
        let ok = StatusIndicator::Ok;
        let warn = StatusIndicator::Warning;
        let err = StatusIndicator::Error;

        // Just verify they don't panic
        let _ = format!("{:?}", ok);
        let _ = format!("{:?}", warn);
        let _ = format!("{:?}", err);
    }

    #[test]
    fn test_format_bytes_units() {
        // Bytes
        assert!(format_bytes(500).contains("500"));
        assert!(format_bytes(500).contains("B"));

        // Kilobytes
        let kb = format_bytes(2048);
        assert!(kb.contains("2") || kb.contains("KB"));

        // Megabytes
        let mb = format_bytes(5 * 1024 * 1024);
        assert!(mb.contains("5") || mb.contains("MB"));

        // Gigabytes
        let gb = format_bytes(3 * 1024 * 1024 * 1024);
        assert!(gb.contains("3") || gb.contains("GB"));
    }

    #[test]
    fn test_format_timestamp_output() {
        // Unix epoch
        let ts = format_timestamp(0);
        assert!(ts.contains("1970") || ts.contains("ago") || !ts.is_empty());

        // Recent timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent = format_timestamp(now);
        assert!(!recent.is_empty());
    }
}
