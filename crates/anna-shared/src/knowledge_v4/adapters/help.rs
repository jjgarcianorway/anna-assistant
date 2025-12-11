//! Command help adapter (v0.0.424).
//!
//! Extracts knowledge from command --help / -h output.

use std::process::Command;

use super::{extract_context, truncate_text, looks_like_docs};
use crate::knowledge_v4::snippet::KnowledgeSnippet;
use crate::knowledge_v4::query::KnowledgeSource;

/// Help output adapter
pub struct HelpAdapter {
    /// Maximum characters per excerpt
    max_chars: usize,
    /// Timeout for help command in ms
    timeout_ms: u64,
}

impl Default for HelpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpAdapter {
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

    /// Query help for a command
    pub fn query(&self, command: &str, topic: Option<&str>) -> Option<KnowledgeSnippet> {
        // Validate command name
        if !is_safe_command(command) {
            return None;
        }

        // Skip dangerous commands
        if is_dangerous_for_help(command) {
            return None;
        }

        // Try --help first
        let content = fetch_help(command, "--help")
            .or_else(|| fetch_help(command, "-h"))?;

        if !looks_like_docs(&content) {
            return None;
        }

        // Extract relevant excerpt
        let excerpt = if let Some(topic) = topic {
            self.extract_topic_excerpt(&content, topic)
        } else {
            self.extract_default_excerpt(&content)
        };

        let mut snippet = KnowledgeSnippet::from_help(command, &excerpt);

        // Boost relevance if topic was found
        if topic.is_some() && excerpt.to_lowercase().contains(&topic.unwrap_or("").to_lowercase()) {
            snippet.relevance = 0.8;
        }

        Some(snippet)
    }

    /// Query multiple commands, return first match
    pub fn query_any(&self, commands: &[&str], topic: Option<&str>) -> Option<KnowledgeSnippet> {
        for cmd in commands {
            if let Some(snippet) = self.query(cmd, topic) {
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

    /// Extract default excerpt (usage line + first options)
    fn extract_default_excerpt(&self, content: &str) -> String {
        let mut excerpt = String::new();
        let mut in_options = false;
        let mut option_count = 0;
        let max_options = 10;

        for line in content.lines() {
            let trimmed = line.trim();

            // Always include usage lines
            let lower = trimmed.to_lowercase();
            if lower.starts_with("usage:") || lower.starts_with("usage ")
                || (trimmed.starts_with(char::is_uppercase) && trimmed.contains("usage"))
            {
                excerpt.push_str(trimmed);
                excerpt.push('\n');
                continue;
            }

            // Detect options section
            if lower == "options:" || lower == "options"
                || lower == "flags:" || lower == "arguments:"
            {
                in_options = true;
                excerpt.push_str(trimmed);
                excerpt.push('\n');
                continue;
            }

            // Collect options
            if in_options {
                if trimmed.starts_with('-') || trimmed.starts_with("  -") {
                    excerpt.push_str(trimmed);
                    excerpt.push('\n');
                    option_count += 1;

                    if option_count >= max_options {
                        excerpt.push_str("  ...\n");
                        break;
                    }
                } else if !trimmed.is_empty() && !trimmed.starts_with(' ') {
                    // New section, stop
                    break;
                } else if !trimmed.is_empty() {
                    // Option continuation
                    excerpt.push_str(trimmed);
                    excerpt.push('\n');
                }
            }

            // Include description if we haven't found options yet
            if !in_options && !excerpt.is_empty() && excerpt.lines().count() < 5 {
                if !trimmed.is_empty() {
                    excerpt.push_str(trimmed);
                    excerpt.push('\n');
                }
            }
        }

        // If no options found, just take first N lines
        if excerpt.is_empty() || excerpt.len() < 50 {
            excerpt = content.lines()
                .take(20)
                .collect::<Vec<_>>()
                .join("\n");
        }

        truncate_text(excerpt.trim(), self.max_chars)
    }
}

/// Fetch help output from a command
fn fetch_help(command: &str, flag: &str) -> Option<String> {
    let output = Command::new(command)
        .arg(flag)
        .output()
        .ok()?;

    // Help might go to stdout or stderr
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Prefer stdout if it has content
    if stdout.len() > stderr.len() && stdout.len() > 50 {
        Some(stdout)
    } else if stderr.len() > 50 {
        Some(stderr)
    } else if output.status.success() && !stdout.is_empty() {
        Some(stdout)
    } else {
        None
    }
}

/// Validate command name is safe
fn is_safe_command(cmd: &str) -> bool {
    !cmd.is_empty()
        && cmd.len() < 100
        && cmd.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !cmd.contains("..")
}

/// Check if command is dangerous to run with --help
fn is_dangerous_for_help(cmd: &str) -> bool {
    const DANGEROUS: &[&str] = &[
        "rm", "dd", "mkfs", "fdisk", "parted", "shred", "wipefs",
        "halt", "poweroff", "reboot", "shutdown",
        "kill", "pkill", "killall",
    ];
    DANGEROUS.contains(&cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_command() {
        assert!(is_safe_command("pacman"));
        assert!(is_safe_command("systemctl"));
        assert!(!is_safe_command("rm -rf /"));
        assert!(!is_safe_command(""));
    }

    #[test]
    fn test_is_dangerous_for_help() {
        assert!(is_dangerous_for_help("rm"));
        assert!(is_dangerous_for_help("dd"));
        assert!(!is_dangerous_for_help("pacman"));
        assert!(!is_dangerous_for_help("ls"));
    }

    #[test]
    fn test_help_adapter_query() {
        let adapter = HelpAdapter::new();
        // Test with a common command
        let snippet = adapter.query("ls", None);
        if snippet.is_some() {
            let s = snippet.unwrap();
            assert_eq!(s.citation_id, "help:ls");
            assert!(s.excerpt.len() > 10);
        }
    }
}
