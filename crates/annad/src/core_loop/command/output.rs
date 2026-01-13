//! Output processing utilities for command results.

/// Strip ANSI escape codes from text
pub fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// v0.0.938: Truncate long command output while preserving useful parts
/// Keeps the first and last portions, with a truncation marker in the middle
pub fn truncate_output(output: &str, max_lines: usize, max_chars: usize) -> String {
    let trimmed = output.trim();

    // Check character limit first (more important for LLM context)
    if trimmed.len() <= max_chars {
        // Still check line limit
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() <= max_lines {
            return trimmed.to_string();
        }
        // Truncate by lines
        let keep_start = max_lines * 2 / 3;
        let keep_end = max_lines - keep_start;
        let start_lines = lines[..keep_start].join("\n");
        let end_lines = lines[lines.len() - keep_end..].join("\n");
        return format!(
            "{}\n\n[... {} lines truncated ...]\n\n{}",
            start_lines,
            lines.len() - max_lines,
            end_lines
        );
    }

    // Truncate by characters
    let lines: Vec<&str> = trimmed.lines().collect();
    let total_lines = lines.len();

    // Keep ~60% from start, ~40% from end
    let keep_start_chars = max_chars * 6 / 10;
    let keep_end_chars = max_chars - keep_start_chars;

    // Find line boundaries
    let mut start_end = 0;
    let mut char_count = 0;
    for (i, line) in lines.iter().enumerate() {
        char_count += line.len() + 1; // +1 for newline
        if char_count >= keep_start_chars {
            start_end = i + 1;
            break;
        }
    }

    let mut end_start = total_lines;
    char_count = 0;
    for (i, line) in lines.iter().rev().enumerate() {
        char_count += line.len() + 1;
        if char_count >= keep_end_chars {
            end_start = total_lines - i - 1;
            break;
        }
    }

    // Ensure we don't overlap
    if end_start <= start_end {
        end_start = start_end + 1;
    }

    if end_start >= total_lines {
        // Just truncate from the end
        let start_lines = lines[..start_end.min(total_lines)].join("\n");
        return format!(
            "{}\n\n[... output truncated ({} chars) ...]",
            start_lines,
            trimmed.len() - start_lines.len()
        );
    }

    let start_portion = lines[..start_end].join("\n");
    let end_portion = lines[end_start..].join("\n");
    let omitted_lines = end_start - start_end;
    let omitted_chars = trimmed.len() - start_portion.len() - end_portion.len();

    format!(
        "{}\n\n[... {} lines / {} chars truncated ...]\n\n{}",
        start_portion, omitted_lines, omitted_chars, end_portion
    )
}

/// v0.0.925: Get alternative command when primary returns empty output
pub fn get_alternative_command(cmd: &str) -> Option<String> {
    let cmd_lower = cmd.to_lowercase();

    // systemctl alternatives
    if cmd_lower.contains("systemctl list-units") && cmd_lower.contains("--failed") {
        return Some("systemctl --failed 2>/dev/null || journalctl -p err -n 5".to_string());
    }

    // Process listing alternatives
    if cmd_lower.starts_with("pgrep ") {
        let pattern = cmd_lower.strip_prefix("pgrep ").unwrap_or("");
        return Some(format!(
            "ps aux | grep -i '{}' | grep -v grep",
            pattern.trim()
        ));
    }

    // Network alternatives
    if cmd_lower.starts_with("ss ") {
        return Some(cmd.replace("ss ", "netstat "));
    }
    if cmd_lower.starts_with("ip addr") {
        return Some("ifconfig 2>/dev/null || hostname -I".to_string());
    }

    // Disk alternatives
    if cmd_lower.starts_with("lsblk") && cmd_lower.contains("-f") {
        return Some("blkid 2>/dev/null || df -Th".to_string());
    }

    // Memory alternatives
    if cmd_lower.starts_with("free ") {
        return Some("cat /proc/meminfo | head -10".to_string());
    }

    // Log alternatives
    if cmd_lower.starts_with("journalctl") && cmd_lower.contains("-p err") {
        return Some("dmesg --level=err,warn 2>/dev/null | tail -20".to_string());
    }

    None
}
