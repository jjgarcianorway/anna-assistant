//! Documentation index for storage and retrieval (v0.0.429).
//!
//! Lightweight on-disk index for full-text search.

use super::{DocSnippet, DocSourceKind, MAX_SNIPPET_SIZE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Documentation index (in-memory + on-disk)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DocIndex {
    /// All snippets by ID
    snippets: HashMap<String, DocSnippet>,
    /// Keyword index (keyword -> snippet IDs)
    keyword_index: HashMap<String, Vec<String>>,
    /// Source index (source kind -> snippet IDs)
    source_index: HashMap<String, Vec<String>>,
    /// Name index (name -> snippet IDs)
    name_index: HashMap<String, Vec<String>>,
    /// Index version for migrations
    version: u32,
    /// Last rebuild timestamp
    last_rebuild: u64,
}

impl DocIndex {
    /// Current index version
    const VERSION: u32 = 1;

    /// Create new empty index
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            ..Default::default()
        }
    }

    /// Load index from disk
    pub fn load(path: &Path) -> Result<Self, IndexError> {
        let index_file = path.join("doc_index.json");

        if !index_file.exists() {
            return Ok(Self::new());
        }

        let content =
            fs::read_to_string(&index_file).map_err(|e| IndexError::IoError(e.to_string()))?;

        let index: Self =
            serde_json::from_str(&content).map_err(|e| IndexError::ParseError(e.to_string()))?;

        // Version check
        if index.version != Self::VERSION {
            return Err(IndexError::VersionMismatch {
                expected: Self::VERSION,
                found: index.version,
            });
        }

        Ok(index)
    }

    /// Save index to disk
    pub fn save(&self, path: &Path) -> Result<(), IndexError> {
        fs::create_dir_all(path).map_err(|e| IndexError::IoError(e.to_string()))?;

        let index_file = path.join("doc_index.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| IndexError::ParseError(e.to_string()))?;

        fs::write(&index_file, content).map_err(|e| IndexError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Add a snippet to the index
    pub fn add(&mut self, mut snippet: DocSnippet) {
        // Truncate content if too large
        snippet.truncate_content(MAX_SNIPPET_SIZE);

        let id = snippet.id.clone();

        // Update keyword index
        for keyword in &snippet.keywords {
            self.keyword_index
                .entry(keyword.clone())
                .or_default()
                .push(id.clone());
        }

        // Update source index
        let source_key = format!("{:?}", snippet.source);
        self.source_index
            .entry(source_key)
            .or_default()
            .push(id.clone());

        // Update name index
        self.name_index
            .entry(snippet.name.to_lowercase())
            .or_default()
            .push(id.clone());

        // Store snippet
        self.snippets.insert(id, snippet);
    }

    /// Remove a snippet by ID
    pub fn remove(&mut self, id: &str) -> Option<DocSnippet> {
        if let Some(snippet) = self.snippets.remove(id) {
            // Clean up keyword index
            for keyword in &snippet.keywords {
                if let Some(ids) = self.keyword_index.get_mut(keyword) {
                    ids.retain(|i| i != id);
                }
            }

            // Clean up source index
            let source_key = format!("{:?}", snippet.source);
            if let Some(ids) = self.source_index.get_mut(&source_key) {
                ids.retain(|i| i != id);
            }

            // Clean up name index
            if let Some(ids) = self.name_index.get_mut(&snippet.name.to_lowercase()) {
                ids.retain(|i| i != id);
            }

            Some(snippet)
        } else {
            None
        }
    }

    /// Get snippet by ID
    pub fn get(&self, id: &str) -> Option<&DocSnippet> {
        self.snippets.get(id)
    }

    /// Get mutable snippet by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut DocSnippet> {
        self.snippets.get_mut(id)
    }

    /// Check if snippet exists
    pub fn contains(&self, id: &str) -> bool {
        self.snippets.contains_key(id)
    }

    /// Search by keywords
    pub fn search(&self, query: &str, sources: &[DocSourceKind], limit: usize) -> Vec<DocSnippet> {
        let query_words: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Score each snippet
        let mut scored: Vec<(u8, &DocSnippet)> = Vec::new();

        for snippet in self.snippets.values() {
            // Filter by source if specified
            if !sources.is_empty() && !sources.contains(&snippet.source) {
                continue;
            }

            let score = self.score_match(snippet, &query_words);
            if score > 0 {
                scored.push((score, snippet));
            }
        }

        // Sort by score (descending), then by source priority
        scored.sort_by(|a, b| match b.0.cmp(&a.0) {
            std::cmp::Ordering::Equal => a.1.source.priority().cmp(&b.1.source.priority()),
            other => other,
        });

        // Return top results with relevance set
        scored
            .into_iter()
            .take(limit)
            .map(|(score, s)| s.clone().with_relevance(score))
            .collect()
    }

    /// Search by exact name
    pub fn search_by_name(&self, name: &str, source: Option<DocSourceKind>) -> Vec<DocSnippet> {
        let name_lower = name.to_lowercase();

        self.name_index
            .get(&name_lower)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.snippets.get(id))
                    .filter(|s| source.map(|src| s.source == src).unwrap_or(true))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all snippets for a source
    pub fn get_by_source(&self, source: DocSourceKind) -> Vec<&DocSnippet> {
        let source_key = format!("{:?}", source);
        self.source_index
            .get(&source_key)
            .map(|ids| ids.iter().filter_map(|id| self.snippets.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get stale snippets that need refresh
    pub fn get_stale(&self, source: DocSourceKind, max_age_days: u64) -> Vec<&DocSnippet> {
        self.get_by_source(source)
            .into_iter()
            .filter(|s| s.is_stale(max_age_days))
            .collect()
    }

    /// Total snippet count
    pub fn len(&self) -> usize {
        self.snippets.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.snippets.is_empty()
    }

    /// Count snippets by source
    pub fn count_by_source(&self) -> HashMap<DocSourceKind, usize> {
        let mut counts = HashMap::new();
        for snippet in self.snippets.values() {
            *counts.entry(snippet.source).or_insert(0) += 1;
        }
        counts
    }

    /// Clear all snippets
    pub fn clear(&mut self) {
        self.snippets.clear();
        self.keyword_index.clear();
        self.source_index.clear();
        self.name_index.clear();
    }

    /// Mark index as rebuilt
    pub fn mark_rebuilt(&mut self) {
        self.last_rebuild = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Calculate relevance score for a snippet
    fn score_match(&self, snippet: &DocSnippet, query_words: &[String]) -> u8 {
        let mut score: u32 = 0;

        for word in query_words {
            // Exact name match: +40
            if snippet.name.to_lowercase() == *word {
                score += 40;
            }
            // Name contains word: +20
            else if snippet.name.to_lowercase().contains(word) {
                score += 20;
            }

            // Keyword match: +15
            if snippet.keywords.iter().any(|k| k == word) {
                score += 15;
            }

            // Summary contains word: +10
            if snippet.summary.to_lowercase().contains(word) {
                score += 10;
            }

            // Content contains word: +5
            if snippet.content.to_lowercase().contains(word) {
                score += 5;
            }
        }

        // Bonus for section match
        if let Some(ref section) = snippet.section {
            let section_lower = section.to_lowercase();
            for word in query_words {
                if section_lower.contains(word) {
                    score += 10;
                }
            }
        }

        // Normalize to 0-100
        (score.min(100)) as u8
    }
}

/// Index errors
#[derive(Debug, Clone)]
pub enum IndexError {
    IoError(String),
    ParseError(String),
    VersionMismatch { expected: u32, found: u32 },
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "Index version mismatch: expected {}, found {}",
                    expected, found
                )
            }
        }
    }
}

impl std::error::Error for IndexError {}

/// Get the doc storage path (tries system, falls back to user)
pub fn get_storage_path() -> PathBuf {
    let system_path = PathBuf::from(super::DOC_STORAGE_PATH);
    if system_path.exists() || fs::create_dir_all(&system_path).is_ok() {
        return system_path;
    }

    // Fall back to user directory
    if let Some(home) = dirs::home_dir() {
        let user_path = home.join(".anna").join("docs");
        let _ = fs::create_dir_all(&user_path);
        return user_path;
    }

    system_path
}

/// Get the wiki cache path
pub fn get_wiki_cache_path() -> PathBuf {
    let system_path = PathBuf::from(super::WIKI_CACHE_PATH);
    if system_path.exists() {
        return system_path;
    }

    // Fall back to user directory
    if let Some(home) = dirs::home_dir() {
        let user_path = home.join(".anna").join("wiki-cache");
        if user_path.exists() {
            return user_path;
        }
    }

    system_path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snippet(name: &str, source: DocSourceKind) -> DocSnippet {
        DocSnippet::new(
            source,
            name,
            Some("1"),
            &format!("{} command description", name),
            &format!("Content about {} and how to use it.", name),
        )
    }

    #[test]
    fn test_index_add_and_get() {
        let mut index = DocIndex::new();
        let snippet = make_snippet("systemctl", DocSourceKind::ManPage);
        let id = snippet.id.clone();

        index.add(snippet);

        assert!(index.contains(&id));
        assert_eq!(index.len(), 1);

        let retrieved = index.get(&id).unwrap();
        assert_eq!(retrieved.name, "systemctl");
    }

    #[test]
    fn test_index_search() {
        let mut index = DocIndex::new();
        index.add(make_snippet("systemctl", DocSourceKind::ManPage));
        index.add(make_snippet("journalctl", DocSourceKind::ManPage));
        index.add(make_snippet("systemd", DocSourceKind::ArchWiki));

        // Search for "systemctl"
        let results = index.search("systemctl", &[], 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "systemctl");

        // Search for "system" should match multiple
        let results = index.search("system", &[], 5);
        assert!(results.len() >= 2);

        // Filter by source
        let results = index.search("system", &[DocSourceKind::ArchWiki], 5);
        assert!(results.iter().all(|s| s.source == DocSourceKind::ArchWiki));
    }

    #[test]
    fn test_index_search_by_name() {
        let mut index = DocIndex::new();
        index.add(make_snippet("pacman", DocSourceKind::ManPage));
        index.add(make_snippet("pacman", DocSourceKind::ArchWiki));

        let results = index.search_by_name("pacman", None);
        assert_eq!(results.len(), 2);

        let results = index.search_by_name("pacman", Some(DocSourceKind::ManPage));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_index_remove() {
        let mut index = DocIndex::new();
        let snippet = make_snippet("test", DocSourceKind::ManPage);
        let id = snippet.id.clone();

        index.add(snippet);
        assert!(index.contains(&id));

        let removed = index.remove(&id);
        assert!(removed.is_some());
        assert!(!index.contains(&id));
    }

    #[test]
    fn test_count_by_source() {
        let mut index = DocIndex::new();
        index.add(make_snippet("cmd1", DocSourceKind::ManPage));
        index.add(make_snippet("cmd2", DocSourceKind::ManPage));
        index.add(make_snippet("topic1", DocSourceKind::ArchWiki));

        let counts = index.count_by_source();
        assert_eq!(counts.get(&DocSourceKind::ManPage), Some(&2));
        assert_eq!(counts.get(&DocSourceKind::ArchWiki), Some(&1));
    }
}
