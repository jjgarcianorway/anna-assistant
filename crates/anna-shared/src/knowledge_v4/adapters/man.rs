//! Man page adapter (v0.0.424).
//!
//! Extracts knowledge from man pages:
//! - NAME section for overview
//! - SYNOPSIS for usage
//! - DESCRIPTION for details
//! - Section-specific content when requested

use std::process::Command;

use super::{extract_context, looks_like_docs, truncate_text};
use crate::knowledge_v4::query::KnowledgeSource;
use crate::knowledge_v4::snippet::KnowledgeSnippet;

/// Man page adapter
pub struct ManAdapter {
    /// Maximum characters per excerpt
    max_chars: usize,
    /// Timeout for man command in ms
    timeout_ms: u64,
}

impl Default for ManAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ManAdapter {
    /// Create new adapter
    pub fn new() -> Self {
        Self {
            max_chars: 1500,
            timeout_ms: 5000,
        }
    }

    /// Set max chars
    pub fn with_max_chars(mut self, max: usize) -> Self {
        self.max_chars = max;
        self
    }

    /// Query man page for a command
    pub fn query(
        &self,
        command: &str,
        topic: Option<&str>,
        section: Option<u8>,
    ) -> Option<KnowledgeSnippet> {
        // Validate command name
        if !is_safe_command(command) {
            return None;
        }

        // Check if man page exists
        if !man_exists(command, section) {
            return None;
        }

        // Get man page content
        let content = fetch_man_content(command, section)?;

        if !looks_like_docs(&content) {
            return None;
        }

        // Extract relevant excerpt
        let excerpt = if let Some(topic) = topic {
            self.extract_topic_excerpt(&content, topic)
        } else {
            self.extract_default_excerpt(&content)
        };

        let mut snippet = KnowledgeSnippet::from_man(command, &excerpt);

        // Add section info if specified
        if let Some(sec) = section {
            snippet.citation_id = format!("man:{}({})", command, sec);
            snippet.title = format!("man {}({})", command, sec);
        }

        Some(snippet)
    }

    /// Query multiple commands, return first match
    pub fn query_any(&self, commands: &[&str], topic: Option<&str>) -> Option<KnowledgeSnippet> {
        for cmd in commands {
            if let Some(snippet) = self.query(cmd, topic, None) {
                return Some(snippet);
            }
        }
        None
    }

    /// Extract excerpt focused on a specific topic
    fn extract_topic_excerpt(&self, content: &str, topic: &str) -> String {
        // First try to find context around the topic
        if let Some(ctx) = extract_context(content, topic, 5) {
            return truncate_text(&ctx, self.max_chars);
        }

        // Fall back to default excerpt
        self.extract_default_excerpt(content)
    }

    /// Extract default excerpt (NAME, SYNOPSIS, start of DESCRIPTION)
    fn extract_default_excerpt(&self, content: &str) -> String {
        let mut excerpt = String::new();
        let mut current_section = String::new();
        let mut in_wanted_section = false;
        let mut lines_collected = 0;

        let wanted_sections = ["NAME", "SYNOPSIS", "DESCRIPTION"];

        for line in content.lines() {
            // Detect section headers
            let trimmed = line.trim();
            if is_section_header(trimmed) {
                current_section = trimmed.to_uppercase();
                in_wanted_section = wanted_sections.contains(&current_section.as_str());

                if in_wanted_section && !excerpt.is_empty() {
                    excerpt.push('\n');
                }
                continue;
            }

            // Collect lines from wanted sections
            if in_wanted_section {
                // Skip empty lines at section start
                if excerpt.is_empty() && trimmed.is_empty() {
                    continue;
                }

                excerpt.push_str(trimmed);
                excerpt.push('\n');
                lines_collected += 1;

                // Limit lines per section
                let max_lines = match current_section.as_str() {
                    "NAME" => 3,
                    "SYNOPSIS" => 5,
                    "DESCRIPTION" => 15,
                    _ => 10,
                };

                if lines_collected > max_lines {
                    in_wanted_section = false;
                    lines_collected = 0;
                }
            }

            // Stop after DESCRIPTION
            if current_section == "DESCRIPTION" && !in_wanted_section {
                break;
            }
        }

        truncate_text(excerpt.trim(), self.max_chars)
    }
}

/// Check if man page exists for a command
fn man_exists(command: &str, section: Option<u8>) -> bool {
    let mut args = vec!["-w"];
    let section_str;
    if let Some(sec) = section {
        section_str = sec.to_string();
        args.push(&section_str);
    }
    args.push(command);

    Command::new("man")
        .args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Fetch man page content as plain text
fn fetch_man_content(command: &str, section: Option<u8>) -> Option<String> {
    let mut args = vec!["-P", "cat"];
    let section_str;
    if let Some(sec) = section {
        section_str = sec.to_string();
        args.push(&section_str);
    }
    args.push(command);

    let output = Command::new("man")
        .args(&args)
        .env("MANWIDTH", "80")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

/// Check if line is a man page section header
fn is_section_header(line: &str) -> bool {
    !line.is_empty()
        && line.len() < 30
        && line
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_whitespace())
        && !line.starts_with(' ')
}

/// Validate command name is safe
fn is_safe_command(cmd: &str) -> bool {
    !cmd.is_empty()
        && cmd.len() < 100
        && cmd
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !cmd.contains("..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_command() {
        assert!(is_safe_command("vim"));
        assert!(is_safe_command("systemctl"));
        assert!(is_safe_command("python3.11"));
        assert!(!is_safe_command("rm -rf /"));
        assert!(!is_safe_command("; rm -rf /"));
        assert!(!is_safe_command(""));
    }

    #[test]
    fn test_is_section_header() {
        assert!(is_section_header("NAME"));
        assert!(is_section_header("SYNOPSIS"));
        assert!(is_section_header("DESCRIPTION"));
        assert!(!is_section_header("   NAME"));
        assert!(!is_section_header("Name of something"));
    }

    #[test]
    fn test_man_adapter_query() {
        let adapter = ManAdapter::new();
        // This will only pass if 'ls' has a man page
        if man_exists("ls", None) {
            let snippet = adapter.query("ls", None, None);
            assert!(snippet.is_some());
            let s = snippet.unwrap();
            assert_eq!(s.citation_id, "man:ls");
        }
    }
}
