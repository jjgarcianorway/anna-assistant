//! Man page fetching functionality.

use crate::evidence_engine::{DocSnippet, DocSource};
use std::process::Command;

/// Max snippet length
const MAX_SNIPPET: usize = 500;

/// Man page line limit
const MAN_LINE_LIMIT: usize = 100;

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
                super::utils::shell_escape(clean_topic),
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
        super::utils::truncate(&content, MAX_SNIPPET)
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

    (name_line, super::utils::truncate(&description, MAX_SNIPPET))
}
