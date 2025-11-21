//! Output Normalization Module (Beta.210)
//!
//! Provides unified normalization for CLI and TUI output following the
//! canonical [SUMMARY]/[DETAILS]/[COMMANDS] format defined in ANSWER_FORMAT.md.
//!
//! Philosophy:
//! - Single source of truth for output format
//! - Semantic structure enforced at normalization layer
//! - Terminal formatting applied in rendering layer only
//! - CLI and TUI share identical semantic content

use owo_colors::OwoColorize;

/// Normalize text for CLI output
///
/// Takes canonical [SUMMARY]/[DETAILS]/[COMMANDS] formatted text and applies
/// terminal formatting for CLI display.
///
/// Features (Beta.216):
/// - Section headers highlighted in cyan+bold
/// - Commands highlighted in green
/// - **bold** markdown converted to ANSI bold
/// - Triple backticks (```) stripped
/// - Preserves semantic structure
/// - Adds terminal colors where supported
pub fn normalize_for_cli(text: &str) -> String {
    let mut output = String::new();
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Beta.216: Handle code block markers (strip them)
        if trimmed == "```bash" || trimmed == "```" {
            in_code_block = !in_code_block;
            continue; // Skip the marker line
        }

        // Section markers: [SUMMARY], [DETAILS], [COMMANDS]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            output.push_str(&format!("{}\n", line.cyan().bold()));
        }
        // Command lines (starting with $ or # or inside code blocks)
        else if trimmed.starts_with('$') || trimmed.starts_with('#') || in_code_block {
            output.push_str(&format!("{}\n", line.green()));
        }
        // Regular content - convert **bold** to ANSI bold
        else {
            let formatted = convert_markdown_to_ansi(line);
            output.push_str(&format!("{}\n", formatted));
        }
    }

    output
}

/// Convert markdown formatting to ANSI terminal codes
///
/// Beta.216: Converts **bold** markdown to ANSI bold sequences
fn convert_markdown_to_ansi(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut in_bold = false;

    while let Some(ch) = chars.next() {
        if ch == '*' && chars.peek() == Some(&'*') {
            // Found ** - toggle bold
            chars.next(); // consume second *

            if in_bold {
                // Ending bold
                result.push_str("\x1b[0m"); // Reset
                in_bold = false;
            } else {
                // Starting bold
                result.push_str("\x1b[1m"); // Bold
                in_bold = true;
            }
        } else {
            result.push(ch);
        }
    }

    // If still in bold at end, reset
    if in_bold {
        result.push_str("\x1b[0m");
    }

    result
}

/// Normalize text for TUI output
///
/// Takes canonical [SUMMARY]/[DETAILS]/[COMMANDS] formatted text and prepares
/// it for TUI rendering.
///
/// Features:
/// - Strips section markers for cleaner TUI display
/// - Preserves content structure
/// - Returns plain text for TUI renderer to style
pub fn normalize_for_tui(text: &str) -> String {
    let mut output = String::new();
    let mut in_section = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Section markers - convert to section breaks in TUI
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_section {
                output.push('\n'); // Add spacing between sections
            }
            in_section = true;
            // Don't include the marker itself in TUI
            continue;
        }

        // Include all other content
        output.push_str(line);
        output.push('\n');
    }

    output
}

/// Generate fallback message when startup metadata is unavailable
///
/// Returns canonical [SUMMARY]/[DETAILS]/[COMMANDS] format with recovery guidance.
pub fn generate_fallback_message(error_msg: &str) -> String {
    format!(
        r#"[SUMMARY]
Startup metadata unavailable.

[DETAILS]
{error_msg}

The welcome system requires access to session metadata for system change detection.

[COMMANDS]
$ systemctl status annad
$ journalctl -u annad --no-pager | tail -20
$ annactl status"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_for_cli_preserves_structure() {
        let input = r#"[SUMMARY]
Test summary

[DETAILS]
Test details

[COMMANDS]
$ echo hello"#;

        let output = normalize_for_cli(input);

        // Should contain all sections
        assert!(output.contains("[SUMMARY]"));
        assert!(output.contains("[DETAILS]"));
        assert!(output.contains("[COMMANDS]"));
        assert!(output.contains("Test summary"));
        assert!(output.contains("$ echo hello"));
    }

    #[test]
    fn test_normalize_for_tui_removes_markers() {
        let input = r#"[SUMMARY]
Test summary

[DETAILS]
Test details"#;

        let output = normalize_for_tui(input);

        // Should NOT contain section markers
        assert!(!output.contains("[SUMMARY]"));
        assert!(!output.contains("[DETAILS]"));

        // Should contain content
        assert!(output.contains("Test summary"));
        assert!(output.contains("Test details"));
    }

    #[test]
    fn test_fallback_message_format() {
        let fallback = generate_fallback_message("Test error");

        assert!(fallback.contains("[SUMMARY]"));
        assert!(fallback.contains("[DETAILS]"));
        assert!(fallback.contains("[COMMANDS]"));
        assert!(fallback.contains("Test error"));
        assert!(fallback.contains("systemctl status annad"));
    }
}
