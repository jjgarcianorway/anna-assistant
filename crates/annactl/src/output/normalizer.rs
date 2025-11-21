//! Canonical answer normalization module for Beta.208
//!
//! This module provides a single source of truth for text normalization across all answer types.
//! All answers (deterministic, template, recipe, LLM, fallback) must pass through these functions
//! to ensure byte-for-byte identical formatting (except ANSI codes) in CLI and TUI modes.
//!
//! Key principles:
//! - Single normalization pipeline for all answer text
//! - Consistent [SUMMARY]/[DETAILS]/[COMMANDS] format enforcement
//! - Markdown normalization (whitespace, blank lines, section spacing)
//! - CLI and TUI receive identical markdown (TUI adds ANSI codes only)

use std::env;

/// Normalize answer text for CLI output
///
/// This is the primary normalization function for command-line output.
/// It applies all normalization rules to produce clean, consistent markdown.
///
/// # Arguments
/// * `answer` - Raw answer text from any source (deterministic, LLM, recipe, etc.)
///
/// # Returns
/// Normalized text ready for CLI display
pub fn normalize_for_cli(answer: &str) -> String {
    let debug = env::var("ANNA_DEBUG_NORMALIZATION").is_ok();

    if debug {
        eprintln!("[DEBUG] normalize_for_cli: Input length = {} bytes", answer.len());
    }

    let normalized = normalize_markdown(answer);

    if debug {
        eprintln!("[DEBUG] normalize_for_cli: Output length = {} bytes", normalized.len());
    }

    normalized
}

/// Normalize answer text for TUI output
///
/// This function prepares text for TUI rendering. The TUI renderer will add ANSI codes
/// for syntax highlighting, but the underlying markdown structure must be identical to CLI.
///
/// # Arguments
/// * `answer` - Raw answer text from any source
///
/// # Returns
/// Normalized text ready for TUI display (before ANSI code injection)
pub fn normalize_for_tui(answer: &str) -> String {
    let debug = env::var("ANNA_DEBUG_NORMALIZATION").is_ok();

    if debug {
        eprintln!("[DEBUG] normalize_for_tui: Input length = {} bytes", answer.len());
    }

    // TUI uses the exact same normalization as CLI
    // ANSI codes are added later by the TUI renderer
    let normalized = normalize_markdown(answer);

    if debug {
        eprintln!("[DEBUG] normalize_for_tui: Output length = {} bytes", normalized.len());
    }

    normalized
}

/// Core markdown normalization function
///
/// This function applies all normalization rules:
/// 1. Trim trailing whitespace from each line
/// 2. Collapse multiple consecutive blank lines into one
/// 3. Remove leading and trailing blank lines
/// 4. Normalize section spacing ([SUMMARY], [DETAILS], [COMMANDS])
/// 5. Ensure proper spacing around code blocks
///
/// # Arguments
/// * `text` - Raw text to normalize
///
/// # Returns
/// Normalized markdown text
fn normalize_markdown(text: &str) -> String {
    let debug = env::var("ANNA_DEBUG_NORMALIZATION").is_ok();

    if debug {
        eprintln!("[DEBUG] normalize_markdown: Starting normalization");
    }

    // Step 1: Trim trailing whitespace from each line
    let lines: Vec<&str> = text.lines().collect();
    let trimmed_lines: Vec<String> = lines.iter().map(|line| line.trim_end().to_string()).collect();

    if debug {
        eprintln!("[DEBUG] normalize_markdown: Trimmed {} lines", trimmed_lines.len());
    }

    // Step 2: Normalize blocks (sections, code blocks, blank lines)
    let normalized_blocks = normalize_blocks(&trimmed_lines.join("\n"));

    if debug {
        eprintln!("[DEBUG] normalize_markdown: Blocks normalized");
    }

    normalized_blocks
}

/// Normalize section blocks, code blocks, and blank line spacing
///
/// This function handles:
/// - [SUMMARY], [DETAILS], [COMMANDS] section spacing
/// - Code block (```) spacing
/// - Collapsing multiple consecutive blank lines
/// - Removing leading/trailing blank lines
///
/// # Arguments
/// * `text` - Text with trimmed lines
///
/// # Returns
/// Text with normalized block spacing
fn normalize_blocks(text: &str) -> String {
    let debug = env::var("ANNA_DEBUG_NORMALIZATION").is_ok();

    let lines: Vec<&str> = text.lines().collect();
    let mut normalized: Vec<String> = Vec::new();
    let mut prev_blank = false;
    let mut in_code_block = false;

    for line in &lines {
        let trimmed = line.trim_end();
        let is_blank = trimmed.is_empty();
        let is_section_header = trimmed.starts_with("[SUMMARY]")
            || trimmed.starts_with("[DETAILS]")
            || trimmed.starts_with("[COMMANDS]");
        let is_code_fence = trimmed.starts_with("```");

        // Track code block state
        if is_code_fence {
            in_code_block = !in_code_block;
        }

        // Inside code blocks: preserve all content exactly (including blank lines)
        if in_code_block && !is_code_fence {
            normalized.push(trimmed.to_string());
            prev_blank = false;
            continue;
        }

        // Section headers: ensure exactly one blank line before (except at start)
        if is_section_header {
            if !normalized.is_empty() && !prev_blank {
                normalized.push(String::new());
            }
            normalized.push(trimmed.to_string());
            prev_blank = false;
            continue;
        }

        // Code fence: ensure blank line before opening fence (except at start)
        if is_code_fence && !in_code_block {
            if !normalized.is_empty() && !prev_blank {
                normalized.push(String::new());
            }
            normalized.push(trimmed.to_string());
            in_code_block = true;
            prev_blank = false;
            continue;
        }

        // Closing code fence: add it and ensure blank line after (unless it's the last line)
        if is_code_fence && in_code_block {
            normalized.push(trimmed.to_string());
            prev_blank = true; // Mark as blank to add spacing after code block
            continue;
        }

        // Blank lines: collapse multiple consecutive blank lines into one
        if is_blank {
            if !prev_blank && !normalized.is_empty() {
                normalized.push(String::new());
                prev_blank = true;
            }
            continue;
        }

        // Regular lines: add normally
        normalized.push(trimmed.to_string());
        prev_blank = false;
    }

    // Remove leading blank lines
    while !normalized.is_empty() && normalized[0].is_empty() {
        normalized.remove(0);
    }

    // Remove trailing blank lines
    while !normalized.is_empty() && normalized[normalized.len() - 1].is_empty() {
        normalized.pop();
    }

    if debug {
        eprintln!(
            "[DEBUG] normalize_blocks: {} lines → {} lines",
            lines.len(),
            normalized.len()
        );
    }

    normalized.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_trailing_whitespace() {
        let input = "Line 1   \nLine 2\t\nLine 3";
        let expected = "Line 1\nLine 2\nLine 3";
        assert_eq!(normalize_for_cli(input), expected);
    }

    #[test]
    fn test_collapse_blank_lines() {
        let input = "Line 1\n\n\n\nLine 2";
        let expected = "Line 1\n\nLine 2";
        assert_eq!(normalize_for_cli(input), expected);
    }

    #[test]
    fn test_remove_leading_trailing_blank_lines() {
        let input = "\n\nLine 1\nLine 2\n\n";
        let expected = "Line 1\nLine 2";
        assert_eq!(normalize_for_cli(input), expected);
    }

    #[test]
    fn test_section_header_spacing() {
        let input = "[SUMMARY]\nSummary text\n[DETAILS]\nDetails text\n[COMMANDS]\nCommands text";
        let expected = "[SUMMARY]\nSummary text\n\n[DETAILS]\nDetails text\n\n[COMMANDS]\nCommands text";
        assert_eq!(normalize_for_cli(input), expected);
    }

    #[test]
    fn test_code_block_spacing() {
        let input = "Text before\n```bash\ncommand\n```\nText after";
        let expected = "Text before\n\n```bash\ncommand\n```\n\nText after";
        assert_eq!(normalize_for_cli(input), expected);
    }

    #[test]
    fn test_code_block_preserves_blank_lines() {
        let input = "```bash\nline1\n\nline3\n```";
        let expected = "```bash\nline1\n\nline3\n```";
        assert_eq!(normalize_for_cli(input), expected);
    }

    #[test]
    fn test_cli_tui_consistency() {
        let input = "[SUMMARY]\n  Text with spaces  \n\n\nMore text\n\n";
        let cli_result = normalize_for_cli(input);
        let tui_result = normalize_for_tui(input);
        assert_eq!(cli_result, tui_result, "CLI and TUI normalization must produce identical output");
    }

    #[test]
    fn test_real_world_deterministic_answer() {
        let input = r#"[SUMMARY]
File /usr/bin/ls is owned by an installed package.

[DETAILS]
/usr/bin/ls is owned by coreutils 9.4-1


[COMMANDS]
$ pacman -Qo /usr/bin/ls
"#;
        let expected = r#"[SUMMARY]
File /usr/bin/ls is owned by an installed package.

[DETAILS]
/usr/bin/ls is owned by coreutils 9.4-1

[COMMANDS]
$ pacman -Qo /usr/bin/ls"#;
        assert_eq!(normalize_for_cli(input), expected);
    }
}
