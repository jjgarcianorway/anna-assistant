//! Hollywood UX styling utilities (v0.0.431).
//!
//! Consistent box-drawing and formatting for 80-column terminals.
//! Minimal unicode, no emojis.

/// Box-drawing characters (ASCII-safe)
pub mod box_chars {
    pub const H_LINE: char = '-';
    pub const V_LINE: char = '|';
    pub const CORNER_TL: char = '+';
    pub const CORNER_TR: char = '+';
    pub const CORNER_BL: char = '+';
    pub const CORNER_BR: char = '+';
}

/// Section labels
pub mod labels {
    pub const USER: &str = "[you]";
    pub const ANNA: &str = "[anna]";
    pub const PROBES: &str = "[probes]";
    pub const DEBUG: &str = "[debug]";
    pub const INTERNAL: &str = "--- internal comms ---";
    pub const DEBUG_SECTION: &str = "--- debug ---";
    pub const WORKING: &str = "[working]";
    pub const ERROR: &str = "[error]";
    pub const TIP: &str = "[tip]";
    pub const SYSTEM: &str = "[system]";
}

/// Draw a horizontal separator line
pub fn h_separator(width: usize) -> String {
    box_chars::H_LINE.to_string().repeat(width)
}

/// Draw a header block with user query
pub fn header_block(query: &str, width: usize) -> String {
    let sep = h_separator(width);
    format!(
        "{sep}\n{label} {query}\n{sep}",
        sep = sep,
        label = labels::USER,
        query = truncate_line(query, width - 7)
    )
}

/// Format section header
pub fn section_header(title: &str) -> String {
    format!("\n{}\n", title)
}

/// Format internal comms line
pub fn internal_comm_line(time_secs: f32, staff: &str, message: &str, show_time: bool) -> String {
    if show_time {
        format!("  [{:.1}s] {}: {}", time_secs, staff, message)
    } else {
        format!("  {}: {}", staff, message)
    }
}

/// Format probe summary line
pub fn probe_line(name: &str, status: &str, duration_ms: u64) -> String {
    let status_str = match status {
        "ok" => "ok",
        "failed" => "FAIL",
        "timeout" => "TIMEOUT",
        _ => status,
    };
    format!("  {:24} {:8} ({:>3}ms)", truncate(name, 24), status_str, duration_ms)
}

/// Format evidence footer line
pub fn evidence_footer(sources: &[String]) -> String {
    if sources.is_empty() {
        String::new()
    } else {
        format!("\nEvidence: {}", sources.join(", "))
    }
}

/// Format status footer
pub fn status_footer(
    status: &str,
    confidence: Option<f32>,
    handler: Option<&str>,
    verified: bool,
) -> String {
    let mut parts = vec![status.to_string()];

    if verified {
        parts.push("Based on verified system data".to_string());
    }

    if let Some(conf) = confidence {
        parts.push(format!("Confidence: {:.0}%", conf * 100.0));
    }

    if let Some(handler) = handler {
        parts.push(format!("handled by {}", handler));
    }

    parts.join(" | ")
}

/// Truncate string to max length
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}

/// Truncate line preserving word boundaries
pub fn truncate_line(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }

    let truncated: String = s.chars().take(max - 3).collect();
    if let Some(space_idx) = truncated.rfind(' ') {
        if space_idx > max / 2 {
            return format!("{}...", &s[..space_idx]);
        }
    }
    format!("{}...", truncated)
}

/// Wrap text to width
pub fn wrap_text(text: &str, width: usize, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let effective_width = width.saturating_sub(indent);

    let mut result = String::new();
    for line in text.lines() {
        if line.len() <= effective_width {
            result.push_str(&indent_str);
            result.push_str(line);
            result.push('\n');
        } else {
            // Word wrap
            let mut current_line = String::new();
            for word in line.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= effective_width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    result.push_str(&indent_str);
                    result.push_str(&current_line);
                    result.push('\n');
                    current_line = word.to_string();
                }
            }
            if !current_line.is_empty() {
                result.push_str(&indent_str);
                result.push_str(&current_line);
                result.push('\n');
            }
        }
    }

    result
}

/// Indent all lines
pub fn indent(text: &str, spaces: usize) -> String {
    let indent_str = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{}{}", indent_str, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a key-value pair with alignment
pub fn kv_pair(key: &str, value: &str, key_width: usize) -> String {
    format!("{:width$} {}", key, value, width = key_width)
}

/// Format a table row
pub fn table_row(cells: &[&str], widths: &[usize]) -> String {
    let formatted: Vec<String> = cells
        .iter()
        .zip(widths.iter())
        .map(|(cell, width)| format!("{:width$}", truncate(cell, *width), width = width))
        .collect();
    formatted.join(" | ")
}

/// Format a bullet point
pub fn bullet(text: &str) -> String {
    format!("  * {}", text)
}

/// Format a numbered item
pub fn numbered_item(num: usize, text: &str) -> String {
    format!("  {}) {}", num, text)
}

/// Format time duration for display
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", ms as f64 / 60000.0)
    }
}

/// Format percentage
pub fn format_percent(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

/// Spinner frames for terminal animation
pub mod spinner {
    pub const FRAMES: &[&str] = &["|", "/", "-", "\\"];
    pub const DOTS: &[&str] = &[".", "..", "...", ".."];
    pub const INTERVAL_MS: u64 = 100;
}

/// Get spinner frame for tick count
pub fn spinner_frame(tick: usize) -> &'static str {
    spinner::FRAMES[tick % spinner::FRAMES.len()]
}

/// Format working status with spinner
pub fn working_status(message: &str, elapsed_secs: f32, tick: usize) -> String {
    format!(
        "{} {} {} ({:.1}s)",
        labels::WORKING,
        spinner_frame(tick),
        message,
        elapsed_secs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 2), "hi");
    }

    #[test]
    fn test_h_separator() {
        assert_eq!(h_separator(5), "-----");
        assert_eq!(h_separator(10).len(), 10);
    }

    #[test]
    fn test_header_block() {
        let header = header_block("how much ram?", 40);
        assert!(header.contains("[you]"));
        assert!(header.contains("how much ram?"));
    }

    #[test]
    fn test_probe_line() {
        let line = probe_line("systemd_boot_time", "ok", 132);
        assert!(line.contains("systemd_boot_time"));
        assert!(line.contains("ok"));
        assert!(line.contains("132ms"));
    }

    #[test]
    fn test_status_footer() {
        let footer = status_footer("System Status", Some(0.9), Some("Sofia (Desktop Jr)"), true);
        assert!(footer.contains("90%"));
        assert!(footer.contains("Sofia"));
        assert!(footer.contains("verified"));
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a very long line that should be wrapped at some point for display.";
        let wrapped = wrap_text(text, 30, 2);
        for line in wrapped.lines() {
            assert!(line.len() <= 30, "Line too long: {}", line);
        }
    }
}
