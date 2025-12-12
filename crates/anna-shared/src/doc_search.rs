//! Local documentation search implementation (v0.0.408).
//!
//! Searches local knowledge sources:
//! - Man pages (via `man -k` / apropos)
//! - /usr/share/doc files
//! - Offline Arch Wiki mirror (if present)
//! - Anna's own docs (recipes, handbook)
//!
//! All sources are local - no internet access.

use crate::knowledge_item::{KnowledgeItem, KnowledgeQuery, KnowledgeSourceType};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, warn};

/// Arch Wiki local mirror path
const ARCH_WIKI_PATH: &str = "/var/lib/anna/arch_wiki";

/// Anna docs path
const ANNA_DOCS_PATH: &str = "/var/lib/anna/docs";

/// Max snippet length
const MAX_SNIPPET: usize = 500;

/// Search all knowledge sources based on query
pub fn search_knowledge(query: &KnowledgeQuery) -> Vec<KnowledgeItem> {
    let mut results = vec![];
    let search_all = query.source_types.is_empty();

    // Search man pages
    if search_all || query.source_types.contains(&KnowledgeSourceType::ManPage) {
        let man_results = search_man_pages(&query.keywords, query.max_items);
        results.extend(man_results);
    }

    // Search local docs
    if search_all || query.source_types.contains(&KnowledgeSourceType::LocalDoc) {
        let doc_results = search_local_docs(&query.keywords, &query.tags, query.max_items);
        results.extend(doc_results);
    }

    // Search Arch Wiki local (if present)
    if search_all
        || query
            .source_types
            .contains(&KnowledgeSourceType::ArchWikiLocal)
    {
        let wiki_results = search_arch_wiki_local(&query.keywords, query.max_items);
        results.extend(wiki_results);
    }

    // Search Anna docs
    if search_all || query.source_types.contains(&KnowledgeSourceType::AnnaDoc) {
        let anna_results = search_anna_docs(&query.keywords, query.max_items);
        results.extend(anna_results);
    }

    // Sort by confidence (descending) and deduplicate
    results.sort_by(|a, b| b.confidence.cmp(&a.confidence));
    deduplicate_results(&mut results);

    // Limit total results
    results.truncate(query.max_items);

    debug!(
        "search_knowledge: {} results for keywords {:?}",
        results.len(),
        query.keywords
    );

    results
}

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

/// Search /usr/share/doc for relevant files
pub fn search_local_docs(keywords: &[String], tags: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let mut results = vec![];

    if keywords.is_empty() && tags.is_empty() {
        return results;
    }

    let doc_dirs = ["/usr/share/doc", "/usr/share/help"];

    // Build grep pattern
    let pattern = if !keywords.is_empty() {
        keywords.join("|")
    } else {
        tags.join("|")
    };

    for doc_dir in &doc_dirs {
        if !Path::new(doc_dir).exists() {
            continue;
        }

        // Use grep to find matching files
        let grep_results = grep_directory(doc_dir, &pattern, limit);
        for (path, snippet) in grep_results {
            let title = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "doc".to_string());

            let item =
                KnowledgeItem::from_path(KnowledgeSourceType::LocalDoc, path, title, snippet)
                    .with_tags(tags.to_vec());

            results.push(item);
        }
    }

    results.truncate(limit);
    results
}

/// Search Arch Wiki local mirror (if present)
pub fn search_arch_wiki_local(keywords: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let wiki_path = Path::new(ARCH_WIKI_PATH);

    if !wiki_path.exists() {
        debug!("Arch Wiki local mirror not found at {}", ARCH_WIKI_PATH);
        return vec![];
    }

    if keywords.is_empty() {
        return vec![];
    }

    let pattern = keywords.join("|");
    let grep_results = grep_directory(ARCH_WIKI_PATH, &pattern, limit);

    grep_results
        .into_iter()
        .map(|(path, snippet)| {
            let title = path
                .file_stem()
                .map(|n| format!("Arch Wiki: {}", n.to_string_lossy()))
                .unwrap_or_else(|| "Arch Wiki".to_string());

            KnowledgeItem::from_path(KnowledgeSourceType::ArchWikiLocal, path, title, snippet)
        })
        .collect()
}

/// Search Anna's own documentation
pub fn search_anna_docs(keywords: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let anna_path = Path::new(ANNA_DOCS_PATH);
    let home_anna = dirs::home_dir()
        .map(|h| h.join(".anna").join("docs"))
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let mut results = vec![];

    // Search both system and user anna docs
    for doc_path in [anna_path.to_path_buf(), home_anna] {
        if !doc_path.exists() {
            continue;
        }

        if keywords.is_empty() {
            continue;
        }

        let pattern = keywords.join("|");
        let grep_results = grep_directory(doc_path.to_str().unwrap_or(""), &pattern, limit);

        for (path, snippet) in grep_results {
            let title = path
                .file_stem()
                .map(|n| format!("Anna: {}", n.to_string_lossy()))
                .unwrap_or_else(|| "Anna doc".to_string());

            let item = KnowledgeItem::from_path(KnowledgeSourceType::AnnaDoc, path, title, snippet);

            results.push(item);
        }
    }

    results.truncate(limit);
    results
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

/// Grep a directory for pattern, return matching files with snippets
fn grep_directory(dir: &str, pattern: &str, limit: usize) -> Vec<(PathBuf, String)> {
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

/// Extract a snippet from content around pattern matches
fn extract_snippet(content: &str, pattern: &str, max_len: usize) -> String {
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

/// Truncate to line boundary
fn truncate_to_line_boundary(s: &str, max: usize) -> String {
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

/// Deduplicate results by ID
fn deduplicate_results(results: &mut Vec<KnowledgeItem>) {
    let mut seen = HashSet::new();
    results.retain(|item| seen.insert(item.id.clone()));
}

/// Check if Arch Wiki local mirror is available
pub fn arch_wiki_available() -> bool {
    Path::new(ARCH_WIKI_PATH).exists()
}

/// Suggest a manual Arch Wiki link (for when local is unavailable)
pub fn suggest_arch_wiki_link(topic: &str) -> String {
    let slug = topic
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    format!("https://wiki.archlinux.org/title/{}", slug)
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

    #[test]
    fn test_truncate_to_line_boundary() {
        let text = "line1\nline2\nline3\nline4";
        let truncated = truncate_to_line_boundary(text, 15);

        assert!(truncated.len() <= 18); // 15 + "..."
        assert!(truncated.ends_with("...") || truncated.len() <= 15);
    }

    #[test]
    fn test_suggest_arch_wiki_link() {
        let link = suggest_arch_wiki_link("systemd service");
        assert!(link.contains("wiki.archlinux.org"));
        assert!(link.contains("systemd_service"));
    }

    #[test]
    fn test_knowledge_query() {
        let query = KnowledgeQuery::new()
            .with_keywords(vec!["systemctl".to_string()])
            .with_limit(5);

        assert_eq!(query.keywords, vec!["systemctl"]);
        assert_eq!(query.max_items, 5);
    }
}
