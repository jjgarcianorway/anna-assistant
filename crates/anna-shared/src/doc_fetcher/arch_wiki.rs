//! Arch Wiki cache fetching functionality.

use crate::evidence_engine::{DocSnippet, DocSource};
use std::fs;
use std::path::PathBuf;

/// Max snippet length
const MAX_SNIPPET: usize = 500;

/// Arch Wiki cache paths
const WIKI_CACHE_PATHS: &[&str] = &["/var/lib/anna/wiki-cache", "~/.anna/wiki-cache"];

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
                let snippet = super::utils::extract_relevant_section(&content, topic, MAX_SNIPPET);
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
                    let snippet = super::utils::extract_relevant_section(&content, topic, MAX_SNIPPET);
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
