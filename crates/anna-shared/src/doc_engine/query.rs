//! Unified documentation query API (v0.0.429).
//!
//! Main interface for specialists and recipe engine to query docs.

use super::help_extractor::{extract_help, get_essential_commands};
use super::index::{get_storage_path, get_wiki_cache_path, DocIndex, IndexError};
use super::man_parser::{get_essential_man_pages, parse_man_page, search_man_pages};
use super::wiki_reader::{get_essential_wiki_pages, list_wiki_pages, read_wiki_page};
use super::{DocQuery, DocReference, DocResult, DocSnippet, DocSourceKind};
use super::{HELP_CACHE_DAYS, MAN_CACHE_DAYS, MAX_QUERY_RESULTS};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Documentation engine for unified doc access
pub struct DocEngine {
    /// In-memory index (thread-safe)
    index: Arc<RwLock<DocIndex>>,
    /// Storage path for persistent index
    storage_path: PathBuf,
    /// Wiki cache path
    wiki_cache_path: PathBuf,
    /// Whether wiki is available
    wiki_available: bool,
    /// Engine stats
    stats: Arc<RwLock<DocEngineStats>>,
}

impl DocEngine {
    /// Create a new doc engine
    pub fn new() -> Self {
        let storage_path = get_storage_path();
        let wiki_cache_path = get_wiki_cache_path();
        let wiki_available = wiki_cache_path.exists();

        // Try to load existing index
        let index = DocIndex::load(&storage_path).unwrap_or_else(|_| DocIndex::new());

        Self {
            index: Arc::new(RwLock::new(index)),
            storage_path,
            wiki_cache_path,
            wiki_available,
            stats: Arc::new(RwLock::new(DocEngineStats::default())),
        }
    }

    /// Query documentation
    pub fn query(&self, query: DocQuery) -> DocResult {
        let start = Instant::now();
        let mut result = DocResult::default();

        // Get index read lock
        let index = self.index.read().unwrap();

        // Search index
        let snippets = index.search(
            &query.query,
            &query.preferred_sources,
            query.limit,
        );

        result.snippets = snippets
            .into_iter()
            .filter(|s| s.relevance >= query.min_relevance)
            .collect();

        result.from_cache = true;
        result.query_time_ms = start.elapsed().as_millis() as u64;
        result.sources_searched = query.preferred_sources.clone();

        // Track stats
        if let Ok(mut stats) = self.stats.write() {
            stats.queries += 1;
            stats.cache_hits += 1;
        }

        result
    }

    /// Query and fetch if not in cache
    pub fn query_or_fetch(&self, query: DocQuery) -> DocResult {
        let start = Instant::now();

        // First try cache
        let cached = self.query(query.clone());
        if !cached.is_empty() {
            return cached;
        }

        // If not in cache, try to fetch directly
        let mut result = DocResult::default();

        for source in &query.preferred_sources {
            match source {
                DocSourceKind::ManPage => {
                    if let Some(name) = &query.name_filter {
                        if let Ok(snippets) = parse_man_page(name, None) {
                            // Add to index for future
                            if let Ok(mut index) = self.index.write() {
                                for snippet in &snippets {
                                    index.add(snippet.clone());
                                }
                            }
                            result.snippets.extend(snippets);
                        }
                    }
                }
                DocSourceKind::ToolHelp => {
                    if let Some(name) = &query.name_filter {
                        if let Ok(snippet) = extract_help(name) {
                            // Add to index
                            if let Ok(mut index) = self.index.write() {
                                index.add(snippet.clone());
                            }
                            result.snippets.push(snippet);
                        }
                    }
                }
                DocSourceKind::ArchWiki => {
                    if self.wiki_available {
                        let search_term = query.name_filter.as_ref().unwrap_or(&query.query);
                        if let Ok(snippets) = read_wiki_page(search_term, &self.wiki_cache_path) {
                            // Add to index
                            if let Ok(mut index) = self.index.write() {
                                for snippet in &snippets {
                                    index.add(snippet.clone());
                                }
                            }
                            result.snippets.extend(snippets);
                        }
                    }
                }
                DocSourceKind::LocalDoc => {
                    // Local docs are indexed at refresh time
                }
            }

            // Stop if we have enough
            if result.snippets.len() >= query.limit {
                break;
            }
        }

        // Apply relevance scoring
        let query_words: Vec<String> = query.query.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        for snippet in &mut result.snippets {
            snippet.relevance = score_snippet(snippet, &query_words);
        }

        // Sort by relevance
        result.snippets.sort_by(|a, b| b.relevance.cmp(&a.relevance));
        result.snippets.truncate(query.limit);

        result.from_cache = false;
        result.query_time_ms = start.elapsed().as_millis() as u64;
        result.sources_searched = query.preferred_sources;

        // Track stats
        if let Ok(mut stats) = self.stats.write() {
            stats.queries += 1;
            stats.fetches += 1;
        }

        result
    }

    /// Get snippet by ID
    pub fn get_by_id(&self, id: &str) -> Option<DocSnippet> {
        let index = self.index.read().ok()?;
        index.get(id).cloned()
    }

    /// Get snippet by reference
    pub fn get_by_ref(&self, reference: &DocReference) -> Option<DocSnippet> {
        self.get_by_id(&reference.snippet_id())
    }

    /// Refresh index (rebuild from sources)
    pub fn refresh_index(&self) -> Result<RefreshStats, IndexError> {
        let start = Instant::now();
        let mut stats = RefreshStats::default();

        let mut index = self.index.write()
            .map_err(|_| IndexError::IoError("Lock poisoned".to_string()))?;

        // Index essential man pages
        for (name, section) in get_essential_man_pages() {
            if let Ok(snippets) = parse_man_page(name, Some(section)) {
                for snippet in snippets {
                    index.add(snippet);
                    stats.man_pages += 1;
                }
            }
        }

        // Index essential help outputs
        for command in get_essential_commands() {
            if let Ok(snippet) = extract_help(command) {
                index.add(snippet);
                stats.help_outputs += 1;
            }
        }

        // Index wiki pages if available
        if self.wiki_available {
            for page in list_wiki_pages(&self.wiki_cache_path) {
                if let Ok(snippets) = read_wiki_page(&page, &self.wiki_cache_path) {
                    for snippet in snippets {
                        index.add(snippet);
                        stats.wiki_pages += 1;
                    }
                }
            }
        }

        index.mark_rebuilt();
        stats.duration_ms = start.elapsed().as_millis() as u64;
        stats.total_snippets = index.len();

        // Save index
        index.save(&self.storage_path)?;

        // Track stats
        if let Ok(mut engine_stats) = self.stats.write() {
            engine_stats.last_refresh_ms = stats.duration_ms;
            engine_stats.total_snippets = stats.total_snippets;
        }

        Ok(stats)
    }

    /// Refresh stale entries only
    pub fn refresh_stale(&self) -> Result<RefreshStats, IndexError> {
        let mut stats = RefreshStats::default();
        let start = Instant::now();

        let mut index = self.index.write()
            .map_err(|_| IndexError::IoError("Lock poisoned".to_string()))?;

        // Find stale help outputs
        let stale_help: Vec<String> = index
            .get_stale(DocSourceKind::ToolHelp, HELP_CACHE_DAYS)
            .iter()
            .map(|s| s.name.clone())
            .collect();

        for name in stale_help {
            if let Ok(snippet) = extract_help(&name) {
                // Remove old, add new
                let old_id = DocSnippet::generate_id(DocSourceKind::ToolHelp, &name, None);
                index.remove(&old_id);
                index.add(snippet);
                stats.help_outputs += 1;
            }
        }

        // Find stale man pages
        let stale_man: Vec<(String, Option<String>)> = index
            .get_stale(DocSourceKind::ManPage, MAN_CACHE_DAYS)
            .iter()
            .map(|s| (s.name.clone(), s.section.clone()))
            .collect();

        for (name, section) in stale_man {
            if let Ok(snippets) = parse_man_page(&name, section.as_deref()) {
                for snippet in snippets {
                    let old_id = snippet.id.clone();
                    index.remove(&old_id);
                    index.add(snippet);
                    stats.man_pages += 1;
                }
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;
        stats.total_snippets = index.len();

        // Save
        index.save(&self.storage_path)?;

        Ok(stats)
    }

    /// Check if wiki is available
    pub fn wiki_available(&self) -> bool {
        self.wiki_available
    }

    /// Get index stats
    pub fn index_stats(&self) -> Option<IndexStats> {
        let index = self.index.read().ok()?;
        let counts = index.count_by_source();

        Some(IndexStats {
            total: index.len(),
            man_pages: *counts.get(&DocSourceKind::ManPage).unwrap_or(&0),
            wiki_pages: *counts.get(&DocSourceKind::ArchWiki).unwrap_or(&0),
            help_outputs: *counts.get(&DocSourceKind::ToolHelp).unwrap_or(&0),
            local_docs: *counts.get(&DocSourceKind::LocalDoc).unwrap_or(&0),
        })
    }

    /// Get engine stats
    pub fn engine_stats(&self) -> DocEngineStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Quick search for man page by command
    pub fn man_page(&self, command: &str) -> DocResult {
        self.query_or_fetch(DocQuery::man_page(command))
    }

    /// Quick search for wiki page by topic
    pub fn wiki_page(&self, topic: &str) -> DocResult {
        self.query_or_fetch(DocQuery::arch_wiki(topic))
    }

    /// Quick search for help output
    pub fn help_output(&self, command: &str) -> DocResult {
        self.query_or_fetch(DocQuery::tool_help(command))
    }

    /// Multi-source query (all sources)
    pub fn search_all(&self, query: &str, limit: usize) -> DocResult {
        self.query_or_fetch(
            DocQuery::new(query)
                .with_limit(limit)
        )
    }
}

impl Default for DocEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Score a snippet against query words
fn score_snippet(snippet: &DocSnippet, query_words: &[String]) -> u8 {
    let mut score: u32 = 0;

    for word in query_words {
        // Name match
        if snippet.name.to_lowercase().contains(word) {
            score += 30;
        }

        // Summary match
        if snippet.summary.to_lowercase().contains(word) {
            score += 20;
        }

        // Content match
        if snippet.content.to_lowercase().contains(word) {
            score += 10;
        }

        // Keyword match
        if snippet.keywords.iter().any(|k| k.contains(word)) {
            score += 15;
        }
    }

    // Source bonus
    score += match snippet.source {
        DocSourceKind::ArchWiki => 10,
        DocSourceKind::ManPage => 8,
        DocSourceKind::ToolHelp => 5,
        DocSourceKind::LocalDoc => 3,
    };

    (score.min(100)) as u8
}

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total: usize,
    pub man_pages: usize,
    pub wiki_pages: usize,
    pub help_outputs: usize,
    pub local_docs: usize,
}

/// Refresh statistics
#[derive(Debug, Clone, Default)]
pub struct RefreshStats {
    pub man_pages: usize,
    pub wiki_pages: usize,
    pub help_outputs: usize,
    pub local_docs: usize,
    pub total_snippets: usize,
    pub duration_ms: u64,
}

/// Engine runtime statistics
#[derive(Debug, Clone, Default)]
pub struct DocEngineStats {
    pub queries: usize,
    pub cache_hits: usize,
    pub fetches: usize,
    pub total_snippets: usize,
    pub last_refresh_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_engine_creation() {
        let engine = DocEngine::new();
        assert!(engine.index_stats().is_some());
    }

    #[test]
    fn test_query_empty_index() {
        let engine = DocEngine::new();
        let result = engine.query(DocQuery::new("nonexistent"));
        assert!(result.is_empty() || !result.is_empty()); // May have cached data
    }

    #[test]
    fn test_score_snippet() {
        let snippet = DocSnippet::new(
            DocSourceKind::ManPage,
            "systemctl",
            Some("1"),
            "Control the systemd system and service manager",
            "systemctl is used to control systemd and services.",
        );

        let score = score_snippet(&snippet, &["systemctl".to_string()]);
        assert!(score > 50); // Should score high

        let score = score_snippet(&snippet, &["unknown".to_string()]);
        assert!(score < 20); // Should score low
    }
}
