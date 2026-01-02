//! Man page search functionality.

use std::process::Command;
use tracing::{debug, warn};

use crate::knowledge_item::{KnowledgeItem, KnowledgeSourceType};

use super::constants::MAX_SNIPPET;
use super::utils::truncate_to_line_boundary;

/// Search man pages using apropos/man -k
pub fn search_man_pages(keywords: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let mut results = vec![];

    if keywords.is_empty() {
        return results;
    }

    // Build apropos query
    let query = keywords.join(" ");

    // Run man -k (apropos)
    let output = Command::new("man").args(["-k", &query]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().take(limit * 2) {
                if let Some(item) = parse_apropos_line(line) {
                    results.push(item);
                }
            }
        }
        Ok(_) => {
            // No matches found (exit code 1)
            debug!("man -k returned no results for: {}", query);
        }
        Err(e) => {
            warn!("Failed to run man -k: {}", e);
        }
    }

    // For each result, try to get a snippet from the actual man page
    for item in &mut results {
        if let Some(snippet) = get_man_page_snippet(&item.title, keywords) {
            item.content_snippet = snippet;
        }
    }

    results.truncate(limit);
    results
}

/// Parse an apropos output line
fn parse_apropos_line(line: &str) -> Option<KnowledgeItem> {
    // Format: "command (section) - description"
    // Example: "systemctl (1) - Control the systemd system and service manager"
    let parts: Vec<&str> = line.splitn(2, " - ").collect();
    if parts.len() < 2 {
        return None;
    }

    let name_section = parts[0].trim();
    let description = parts[1].trim();

    // Extract command name (before section)
    let name = name_section
        .split('(')
        .next()
        .map(|s| s.trim())
        .unwrap_or(name_section);

    let title = format!("man {}", name);

    Some(
        KnowledgeItem::new(KnowledgeSourceType::ManPage, title, description)
            .with_tags(vec![name.to_string()]),
    )
}

/// Get a relevant snippet from a man page
fn get_man_page_snippet(title: &str, keywords: &[String]) -> Option<String> {
    // Extract command from "man command" title
    let command = title.strip_prefix("man ").unwrap_or(title);

    // Run man with col to strip formatting
    let output = Command::new("sh")
        .args([
            "-c",
            &format!("man {} 2>/dev/null | col -bx | head -100", command),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // Find a relevant section containing keywords
    let lower_content = content.to_lowercase();
    for keyword in keywords {
        let kw_lower = keyword.to_lowercase();
        if let Some(pos) = lower_content.find(&kw_lower) {
            // Extract context around the keyword
            let start = pos.saturating_sub(100);
            let end = (pos + 400).min(content.len());
            let snippet = &content[start..end];
            return Some(truncate_to_line_boundary(snippet, MAX_SNIPPET));
        }
    }

    // Fallback: return first part of man page
    Some(truncate_to_line_boundary(&content, MAX_SNIPPET))
}

/// Get --help output for a command
pub fn get_help_output(command: &str) -> Option<KnowledgeItem> {
    // Try --help first, then -h
    let output = Command::new(command)
        .arg("--help")
        .output()
        .or_else(|_| Command::new(command).arg("-h").output());

    match output {
        Ok(out) if out.status.success() || !out.stdout.is_empty() => {
            let content = String::from_utf8_lossy(&out.stdout);
            let snippet = truncate_to_line_boundary(&content, MAX_SNIPPET);

            Some(
                KnowledgeItem::new(
                    KnowledgeSourceType::HelpOutput,
                    format!("{} --help", command),
                    snippet,
                )
                .with_tags(vec![command.to_string()]),
            )
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_apropos_line() {
        let line = "systemctl (1) - Control the systemd system and service manager";
        let item = parse_apropos_line(line).unwrap();

        assert_eq!(item.title, "man systemctl");
        assert!(item.content_snippet.contains("Control"));
    }
}
