//! Shared utility functions for documentation search.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::knowledge_item::KnowledgeItem;

use super::constants::MAX_SNIPPET;

/// Truncate to line boundary
pub fn truncate_to_line_boundary(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }

    // Find last newline before max
    let truncated = &s[..max];
    if let Some(pos) = truncated.rfind('\n') {
        format!("{}...", &truncated[..pos])
    } else {
        format!("{}...", truncated)
    }
}

/// Extract a snippet from content around pattern matches
pub fn extract_snippet(content: &str, pattern: &str, max_len: usize) -> String {
    let lower_content = content.to_lowercase();
    let lower_pattern = pattern.to_lowercase();

    // Find first match
    if let Some(pos) = lower_content.find(&lower_pattern) {
        let start = pos.saturating_sub(50);
        let end = (pos + max_len - 50).min(content.len());
        return truncate_to_line_boundary(&content[start..end], max_len);
    }

    // Fallback: beginning of content
    truncate_to_line_boundary(content, max_len)
}

/// Deduplicate results by ID
pub fn deduplicate_results(results: &mut Vec<KnowledgeItem>) {
    let mut seen = HashSet::new();
    results.retain(|item| seen.insert(item.id.clone()));
}

/// Grep a directory for pattern, return matching files with snippets
pub fn grep_directory(dir: &str, pattern: &str, limit: usize) -> Vec<(PathBuf, String)> {
    let mut results = vec![];

    // Try ripgrep first (faster), fall back to grep
    let rg_output = Command::new("rg")
        .args([
            "-l",
            "-i",
            "--max-count",
            "1",
            "--type",
            "txt",
            "--type",
            "md",
            pattern,
            dir,
        ])
        .output();

    let files: Vec<PathBuf> = match rg_output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .take(limit)
            .map(PathBuf::from)
            .collect(),
        _ => {
            // Fallback to grep
            let grep_output = Command::new("grep")
                .args(["-r", "-l", "-i", pattern, dir])
                .output();

            match grep_output {
                Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .take(limit)
                    .map(PathBuf::from)
                    .collect(),
                _ => vec![],
            }
        }
    };

    // Get snippet from each file
    for path in files {
        if let Ok(content) = fs::read_to_string(&path) {
            let snippet = extract_snippet(&content, pattern, MAX_SNIPPET);
            results.push((path, snippet));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_to_line_boundary() {
        let text = "line1\nline2\nline3\nline4";
        let truncated = truncate_to_line_boundary(text, 15);

        assert!(truncated.len() <= 18); // 15 + "..."
        assert!(truncated.ends_with("...") || truncated.len() <= 15);
    }
}
