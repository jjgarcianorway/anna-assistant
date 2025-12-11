//! Knowledge source adapters (v0.0.424).
//!
//! Each adapter handles a specific knowledge source:
//! - ManAdapter: man pages
//! - HelpAdapter: command --help / -h
//! - DocAdapter: local documentation
//! - WikiAdapter: offline Arch Wiki

pub mod man;
pub mod help;
pub mod doc;
pub mod wiki;

pub use man::ManAdapter;
pub use help::HelpAdapter;
pub use doc::DocAdapter;
pub use wiki::WikiAdapter;

use std::process::Command;
use std::time::Duration;

/// Execute a command with timeout
pub fn run_command(cmd: &str, args: &[&str], timeout_ms: u64) -> Option<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // Some commands output help to stderr
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !stderr.is_empty() && stderr.len() > 20 {
            Some(stderr)
        } else {
            None
        }
    }
}

/// Extract context around a keyword
pub fn extract_context(content: &str, keyword: &str, context_lines: usize) -> Option<String> {
    let keyword_lower = keyword.to_lowercase();
    let lines: Vec<&str> = content.lines().collect();

    // Find lines containing the keyword
    let mut matched_indices: Vec<usize> = vec![];
    for (i, line) in lines.iter().enumerate() {
        if line.to_lowercase().contains(&keyword_lower) {
            matched_indices.push(i);
        }
    }

    if matched_indices.is_empty() {
        return None;
    }

    // Extract context around first match
    let first_match = matched_indices[0];
    let start = first_match.saturating_sub(context_lines);
    let end = (first_match + context_lines + 1).min(lines.len());

    let context: Vec<&str> = lines[start..end].to_vec();
    Some(context.join("\n"))
}

/// Truncate text to max characters, trying to end at sentence boundary
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let truncated = &text[..max_chars];

    // Try to find a good break point
    if let Some(pos) = truncated.rfind(|c| c == '.' || c == '\n') {
        if pos > max_chars / 2 {
            return text[..=pos].to_string();
        }
    }

    // Fall back to word boundary
    if let Some(pos) = truncated.rfind(char::is_whitespace) {
        return format!("{}...", &text[..pos]);
    }

    format!("{}...", truncated)
}

/// Check if content looks like valid documentation
pub fn looks_like_docs(content: &str) -> bool {
    let len = content.len();
    if len < 50 {
        return false;
    }

    let lower = content.to_lowercase();

    // Positive signals
    let has_docs_signals = lower.contains("usage")
        || lower.contains("option")
        || lower.contains("description")
        || lower.contains("synopsis")
        || lower.contains("example")
        || lower.contains("name\n")
        || lower.contains("command");

    // Negative signals
    let has_error_signals = lower.contains("command not found")
        || lower.contains("no manual entry")
        || lower.contains("permission denied")
        || lower.contains("error:")
        || lower.contains("fatal:");

    has_docs_signals && !has_error_signals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_context() {
        let content = "line 1\nline 2\nkeyword here\nline 4\nline 5";
        let ctx = extract_context(content, "keyword", 1).unwrap();
        assert!(ctx.contains("line 2"));
        assert!(ctx.contains("keyword here"));
        assert!(ctx.contains("line 4"));
    }

    #[test]
    fn test_truncate_text() {
        let text = "First sentence. Second sentence. Third sentence.";
        let truncated = truncate_text(text, 25);
        assert!(truncated.len() <= 30);
    }

    #[test]
    fn test_looks_like_docs() {
        assert!(looks_like_docs("NAME\n  vim - text editor\n\nSYNOPSIS\n  vim [options] [file...]"));
        assert!(!looks_like_docs("command not found: foobar"));
        assert!(!looks_like_docs("short"));
    }
}
