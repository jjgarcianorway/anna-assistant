//! Text Wrapper v7.28.0 - Zero Truncation Output Formatting
//!
//! Shared formatter for all annactl outputs:
//! - Detects terminal width (fallback to 80)
//! - Wraps on word boundaries when possible
//! - Hard-wraps single tokens that exceed width
//! - Preserves indentation for wrapped lines
//! - Preserves newlines from source content
//! - Strips control characters
//! - Never truncates with "..."

use std::io::IsTerminal;

/// Get terminal width, fallback to 80
pub fn get_terminal_width() -> usize {
    if std::io::stdout().is_terminal() {
        terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(80)
    } else {
        80
    }
}

/// Strip control characters and ANSI escape sequences except newlines and tabs
fn strip_control_chars(s: &str) -> String {
    // First, remove ANSI escape sequences (like \x1b[31m)
    let ansi_re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    let s = ansi_re.replace_all(s, "");

    // Then filter remaining control chars
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

/// Wrap text to fit within width, preserving indentation
///
/// # Arguments
/// * `text` - The text to wrap
/// * `indent` - Number of spaces for first line indent
/// * `subsequent_indent` - Number of spaces for continuation lines
/// * `width` - Maximum line width (0 = use terminal width)
pub fn wrap_text(text: &str, indent: usize, subsequent_indent: usize, width: usize) -> String {
    let width = if width == 0 {
        get_terminal_width()
    } else {
        width
    };
    let text = strip_control_chars(text);

    let mut result = String::new();
    let indent_str: String = " ".repeat(indent);
    let subsequent_str: String = " ".repeat(subsequent_indent);

    for (line_idx, line) in text.lines().enumerate() {
        if line_idx > 0 {
            result.push('\n');
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Detect existing indentation in source
        let source_indent = line.len() - line.trim_start().len();
        let effective_indent = if line_idx == 0 {
            indent
        } else {
            subsequent_indent.max(source_indent)
        };
        let effective_indent_str = " ".repeat(effective_indent);

        let available_width = width.saturating_sub(effective_indent);
        if available_width < 10 {
            // Terminal too narrow, just output as-is
            result.push_str(&effective_indent_str);
            result.push_str(trimmed);
            continue;
        }

        let wrapped = wrap_line(
            trimmed,
            available_width,
            &effective_indent_str,
            &subsequent_str,
        );
        result.push_str(&wrapped);
    }

    result
}

/// Wrap a single line of text
fn wrap_line(text: &str, width: usize, first_indent: &str, subsequent_indent: &str) -> String {
    if text.len() <= width {
        return format!("{}{}", first_indent, text);
    }

    let mut result = String::new();
    let mut current_line = String::new();
    let mut first_line = true;

    for word in text.split_whitespace() {
        if word.len() > width {
            // Word is longer than width, need to hard-wrap
            if !current_line.is_empty() {
                if first_line {
                    result.push_str(first_indent);
                    first_line = false;
                } else {
                    result.push('\n');
                    result.push_str(subsequent_indent);
                }
                result.push_str(&current_line);
                current_line.clear();
            }

            // Hard-wrap the long word
            let mut remaining = word;
            while !remaining.is_empty() {
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                    result.push_str(subsequent_indent);
                } else if first_line {
                    result.push_str(first_indent);
                    first_line = false;
                } else if result.is_empty() {
                    result.push_str(first_indent);
                    first_line = false;
                }

                let chunk_len = remaining.len().min(width);
                result.push_str(&remaining[..chunk_len]);
                remaining = &remaining[chunk_len..];
            }
        } else if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Need to wrap
            if first_line {
                result.push_str(first_indent);
                first_line = false;
            } else {
                result.push('\n');
                result.push_str(subsequent_indent);
            }
            result.push_str(&current_line);
            current_line = word.to_string();
        }
    }

    // Output remaining content
    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
            result.push_str(subsequent_indent);
        } else {
            result.push_str(first_indent);
        }
        result.push_str(&current_line);
    }

    result
}

/// Wrap text with a prefix on the first line and subsequent indent
pub fn wrap_with_prefix(prefix: &str, text: &str, width: usize) -> String {
    let width = if width == 0 {
        get_terminal_width()
    } else {
        width
    };
    let text = strip_control_chars(text);
    let prefix_len = prefix.chars().count();
    let subsequent_indent = " ".repeat(prefix_len);

    let available = width.saturating_sub(prefix_len);
    if available < 10 {
        return format!("{}{}", prefix, text);
    }

    let mut result = String::new();
    let mut first_line = true;

    for (line_idx, line) in text.lines().enumerate() {
        if line_idx > 0 {
            result.push('\n');
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if first_line {
            if trimmed.len() <= available {
                result.push_str(prefix);
                result.push_str(trimmed);
            } else {
                let wrapped = wrap_line(trimmed, available, prefix, &subsequent_indent);
                result.push_str(&wrapped);
            }
            first_line = false;
        } else {
            if trimmed.len() <= available {
                result.push_str(&subsequent_indent);
                result.push_str(trimmed);
            } else {
                let wrapped = wrap_line(trimmed, available, &subsequent_indent, &subsequent_indent);
                result.push_str(&wrapped);
            }
        }
    }

    result
}

/// Format a key-value pair with wrapped value
pub fn format_kv(key: &str, value: &str, key_width: usize, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let padded_key = format!("{:<width$}", key, width = key_width);
    let prefix = format!("{}{}", indent_str, padded_key);
    wrap_with_prefix(&prefix, value, 0)
}

/// Format a list item with wrapped content
pub fn format_list_item(bullet: &str, content: &str, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let prefix = format!("{}{} ", indent_str, bullet);
    wrap_with_prefix(&prefix, content, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_control_chars() {
        let input = "hello\x00world\x1b[31m test\n";
        let result = strip_control_chars(input);
        assert_eq!(result, "helloworld test\n");
    }

    #[test]
    fn test_wrap_short_text() {
        let result = wrap_text("hello world", 2, 4, 80);
        assert_eq!(result, "  hello world");
    }

    #[test]
    fn test_wrap_long_text() {
        let text = "This is a long line that should be wrapped to fit within the specified width";
        let result = wrap_text(text, 2, 4, 40);
        assert!(result.contains('\n'));
        // Each line should be <= 40 chars
        for line in result.lines() {
            assert!(line.len() <= 40, "Line too long: {}", line);
        }
    }

    #[test]
    fn test_no_truncation() {
        let text = "This is a test with no truncation markers anywhere";
        let result = wrap_text(text, 0, 0, 80);
        assert!(!result.contains("..."), "Should not contain truncation");
        assert!(
            result.contains("truncation markers"),
            "Should preserve content"
        );
    }

    #[test]
    fn test_hard_wrap_long_word() {
        let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let result = wrap_text(text, 2, 4, 20);
        // Should be split across multiple lines
        assert!(result.lines().count() > 1);
        // All content preserved
        let joined: String = result.lines().map(|l| l.trim()).collect();
        assert_eq!(joined, text);
    }

    // Snow Leopard v7.28.0 tests

    #[test]
    fn snow_leopard_v728_no_truncation_markers() {
        // v7.28.0: Assert no "..." anywhere in wrapped output
        let long_text =
            "This is a very long message that would previously be truncated with ellipsis \
            but now should be fully preserved with word wrapping applied to fit the terminal width";
        let result = wrap_text(long_text, 2, 4, 60);

        assert!(
            !result.contains("..."),
            "v7.28.0: Should never truncate with '...'"
        );
        assert!(
            result.contains("preserved"),
            "All content must be preserved"
        );
        assert!(result.contains("terminal width"), "No content loss allowed");
    }

    #[test]
    fn snow_leopard_v728_format_kv_preserves_value() {
        let value = "A very long value that should wrap but not be truncated ever";
        let result = format_kv("Key:", &value, 10, 2);

        assert!(
            !result.contains("..."),
            "v7.28.0: KV format must not truncate"
        );
        assert!(result.contains("truncated ever"), "Full value preserved");
    }

    #[test]
    fn snow_leopard_v728_format_list_item_no_truncation() {
        let content = "An extremely long list item content that would previously be cut off \
            but now must wrap properly to preserve all information for the user";
        let result = format_list_item("-", &content, 2);

        assert!(
            !result.contains("..."),
            "v7.28.0: List items must not truncate"
        );
        assert!(result.contains("for the user"), "All content preserved");
    }

    #[test]
    fn snow_leopard_v728_preserves_special_chars() {
        // Test that paths and special chars are preserved
        let text = "/home/user/.config/anna/config.toml (permissions: 644)";
        let result = wrap_text(text, 0, 0, 80);

        assert!(result.contains("/home/user/.config"), "Paths preserved");
        assert!(result.contains("config.toml"), "Filenames preserved");
        assert!(result.contains("644"), "Numbers preserved");
    }

    #[test]
    fn snow_leopard_v728_install_disclosure_message() {
        // Test that install messages are complete
        let msg = "[INSTALLING DEPENDENCY] Package: pciutils v3.9.0 \
            Reason: PCI device enumeration Metrics: pci_devices, gpu_info";
        let result = wrap_text(&msg, 2, 4, 60);

        assert!(!result.contains("..."), "Install messages not truncated");
        assert!(result.contains("pciutils"), "Package name preserved");
        assert!(result.contains("gpu_info"), "Full metrics preserved");
    }
}
