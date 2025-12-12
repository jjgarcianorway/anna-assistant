//! Doc Fetchers - Local documentation sources (v0.0.410).
//!
//! Priority order:
//! 1. Local Arch Wiki cache
//! 2. Man pages
//! 3. --help output
//! 4. /usr/share/doc files
//!
//! All sources are local - no internet access.

use crate::evidence_engine::{DocSnippet, DocSource};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, warn};

/// Max snippet length
const MAX_SNIPPET: usize = 500;

/// Man page line limit
const MAN_LINE_LIMIT: usize = 100;

/// Help output line limit
const HELP_LINE_LIMIT: usize = 40;

/// Arch Wiki cache paths
const WIKI_CACHE_PATHS: &[&str] = &["/var/lib/anna/wiki-cache", "~/.anna/wiki-cache"];

/// Fetch documentation for a list of tags
pub fn fetch_docs(tags: &[String], max_docs: usize) -> Vec<DocSnippet> {
    let mut docs = vec![];

    for tag in tags {
        // Try each source in priority order
        if let Some(doc) = fetch_arch_wiki(tag) {
            docs.push(doc);
        }
        if let Some(doc) = fetch_man_page(tag) {
            docs.push(doc);
        }
        if let Some(doc) = fetch_help_output(tag) {
            docs.push(doc);
        }

        if docs.len() >= max_docs {
            break;
        }
    }

    // Sort by relevance and deduplicate
    docs.sort_by(|a, b| b.relevance.cmp(&a.relevance));
    dedup_docs(&mut docs);
    docs.truncate(max_docs);

    docs
}

/// Fetch from local Arch Wiki cache
pub fn fetch_arch_wiki(topic: &str) -> Option<DocSnippet> {
    let cache_path = find_wiki_cache()?;
    let topic_lower = topic.to_lowercase();

    // Try exact match first
    let candidates = [
        format!("{}.txt", topic),
        format!("{}.md", topic),
        format!("{}.html", topic),
        topic.to_string(),
    ];

    for candidate in &candidates {
        let path = cache_path.join(candidate);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                let snippet = extract_relevant_section(&content, topic, MAX_SNIPPET);
                return Some(
                    DocSnippet::new(
                        DocSource::ArchWiki,
                        &format!("Arch Wiki: {}", topic),
                        &snippet,
                        &path.display().to_string(),
                    )
                    .with_relevance(90),
                );
            }
        }
    }

    // Search in cache directory for partial matches
    if let Ok(entries) = fs::read_dir(&cache_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains(&topic_lower) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let snippet = extract_relevant_section(&content, topic, MAX_SNIPPET);
                    let title = entry
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".txt")
                        .trim_end_matches(".md")
                        .trim_end_matches(".html")
                        .to_string();
                    return Some(
                        DocSnippet::new(
                            DocSource::ArchWiki,
                            &format!("Arch Wiki: {}", title),
                            &snippet,
                            &entry.path().display().to_string(),
                        )
                        .with_relevance(75),
                    );
                }
            }
        }
    }

    None
}

/// Fetch man page for a topic/command
pub fn fetch_man_page(topic: &str) -> Option<DocSnippet> {
    // Clean topic (remove version suffixes, etc.)
    let clean_topic = topic.split('(').next().unwrap_or(topic).trim();

    // Try to get man page
    let output = Command::new("sh")
        .args([
            "-c",
            &format!(
                "MANWIDTH=80 man -P cat {} 2>/dev/null | head -{}",
                shell_escape(clean_topic),
                MAN_LINE_LIMIT
            ),
        ])
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout);
    if content.trim().is_empty() {
        return None;
    }

    // Parse man page structure
    let (name_line, description) = parse_man_page(&content, clean_topic);

    let snippet = if !description.is_empty() {
        description
    } else {
        truncate(&content, MAX_SNIPPET)
    };

    Some(
        DocSnippet::new(
            DocSource::ManPage,
            &format!("man {}", clean_topic),
            &snippet,
            &format!("man {}(1)", clean_topic),
        )
        .with_relevance(85),
    )
}

/// Parse man page to extract NAME and DESCRIPTION
fn parse_man_page(content: &str, topic: &str) -> (String, String) {
    let mut name_line = String::new();
    let mut description = String::new();
    let mut in_description = false;
    let mut desc_lines = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Find NAME section
        if trimmed.contains(" - ") && name_line.is_empty() {
            name_line = trimmed.to_string();
        }

        // Find DESCRIPTION section
        if trimmed == "DESCRIPTION" || trimmed == "Description" {
            in_description = true;
            continue;
        }

        // Collect description lines
        if in_description {
            if trimmed.chars().all(|c| c.is_uppercase() || c == ' ') && trimmed.len() > 3 {
                // New section header, stop
                break;
            }
            if !trimmed.is_empty() {
                description.push_str(trimmed);
                description.push(' ');
                desc_lines += 1;
                if desc_lines >= 10 {
                    break;
                }
            }
        }
    }

    (name_line, truncate(&description, MAX_SNIPPET))
}

/// Fetch --help or -h output for a command
pub fn fetch_help_output(command: &str) -> Option<DocSnippet> {
    // Only try for commands that look safe
    if !is_safe_help_command(command) {
        return None;
    }

    // Try --help first, then -h
    let output = Command::new("sh")
        .args([
            "-c",
            &format!(
                "{} --help 2>&1 | head -{} || {} -h 2>&1 | head -{}",
                shell_escape(command),
                HELP_LINE_LIMIT,
                shell_escape(command),
                HELP_LINE_LIMIT
            ),
        ])
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);
    if content.trim().is_empty() || content.contains("command not found") {
        return None;
    }

    // Check if it looks like help output
    let content_lower = content.to_lowercase();
    if !content_lower.contains("usage")
        && !content_lower.contains("options")
        && !content_lower.contains("--help")
    {
        return None;
    }

    Some(
        DocSnippet::new(
            DocSource::HelpOutput,
            &format!("{} --help", command),
            &truncate(&content, MAX_SNIPPET),
            &format!("{} --help", command),
        )
        .with_relevance(70),
    )
}

/// Check if command is safe to run with --help
fn is_safe_help_command(cmd: &str) -> bool {
    let clean = cmd.split_whitespace().next().unwrap_or(cmd);

    // Blocklist dangerous commands
    let dangerous = [
        "rm", "dd", "mkfs", "fdisk", "parted", "sudo", "su", "chmod", "chown", "kill", "reboot",
        "shutdown", "halt",
    ];

    !dangerous.contains(&clean)
}

/// Search /usr/share/doc for a topic
pub fn fetch_local_doc(topic: &str) -> Option<DocSnippet> {
    let doc_dirs = ["/usr/share/doc", "/usr/share/help"];

    for doc_dir in &doc_dirs {
        let base = Path::new(doc_dir);
        if !base.exists() {
            continue;
        }

        // Look for directories matching topic
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&topic.to_lowercase()) {
                    // Look for README or similar
                    let readme_names = ["README", "README.md", "README.txt", "index.html"];
                    for readme in &readme_names {
                        let readme_path = entry.path().join(readme);
                        if readme_path.exists() {
                            if let Ok(content) = fs::read_to_string(&readme_path) {
                                return Some(
                                    DocSnippet::new(
                                        DocSource::LocalDoc,
                                        &format!("doc: {}", entry.file_name().to_string_lossy()),
                                        &extract_relevant_section(&content, topic, MAX_SNIPPET),
                                        &readme_path.display().to_string(),
                                    )
                                    .with_relevance(60),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Find wiki cache directory
fn find_wiki_cache() -> Option<PathBuf> {
    for path in WIKI_CACHE_PATHS {
        let expanded = if path.starts_with("~") {
            dirs::home_dir()?.join(&path[2..])
        } else {
            PathBuf::from(path)
        };
        if expanded.exists() {
            return Some(expanded);
        }
    }
    None
}

/// Extract section relevant to topic
fn extract_relevant_section(content: &str, topic: &str, max_len: usize) -> String {
    let topic_lower = topic.to_lowercase();
    let content_lower = content.to_lowercase();

    // Find first occurrence of topic
    if let Some(pos) = content_lower.find(&topic_lower) {
        let start = pos.saturating_sub(50);
        let end = (pos + max_len - 50).min(content.len());

        // Extend to line boundaries
        let slice = &content[start..end];
        return truncate_to_lines(slice, max_len);
    }

    // Fallback: return beginning
    truncate_to_lines(content, max_len)
}

/// Truncate to line boundaries
fn truncate_to_lines(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }

    let truncated = &s[..max];
    if let Some(pos) = truncated.rfind('\n') {
        format!("{}...", &truncated[..pos])
    } else {
        format!("{}...", truncated)
    }
}

/// Simple truncate
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Shell escape a string
fn shell_escape(s: &str) -> String {
    // Basic escaping - just alphanumeric and some safe chars
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('\'');
    for c in s.chars() {
        if c == '\'' {
            escaped.push_str("'\"'\"'");
        } else {
            escaped.push(c);
        }
    }
    escaped.push('\'');
    escaped
}

/// Deduplicate docs by title
fn dedup_docs(docs: &mut Vec<DocSnippet>) {
    let mut seen = std::collections::HashSet::new();
    docs.retain(|d| seen.insert(d.title.clone()));
}

/// Check if wiki cache is available
pub fn wiki_cache_available() -> bool {
    find_wiki_cache().is_some()
}

/// Get wiki cache stats
pub fn wiki_cache_stats() -> Option<WikiCacheStats> {
    let cache_path = find_wiki_cache()?;
    let mut stats = WikiCacheStats::default();

    if let Ok(entries) = fs::read_dir(&cache_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                stats.file_count += 1;
                if let Ok(meta) = entry.metadata() {
                    stats.total_size += meta.len();
                }
            }
        }
    }

    stats.path = cache_path.display().to_string();
    Some(stats)
}

/// Wiki cache statistics
#[derive(Debug, Default)]
pub struct WikiCacheStats {
    pub path: String,
    pub file_count: usize,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("test"), "'test'");
        assert_eq!(shell_escape("test's"), "'test'\"'\"'s'");
    }

    #[test]
    fn test_safe_help_command() {
        assert!(is_safe_help_command("pacman"));
        assert!(is_safe_help_command("systemctl"));
        assert!(!is_safe_help_command("rm"));
        assert!(!is_safe_help_command("sudo"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_extract_relevant_section() {
        let content = "First line\nSecond line about vim\nThird line";
        let section = extract_relevant_section(content, "vim", 100);
        assert!(section.contains("vim"));
    }
}
