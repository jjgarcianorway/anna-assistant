//! Doc brain module (v0.0.406).
//!
//! Provides unified document search across all knowledge sources:
//! - Built-in Arch Linux knowledge pack
//! - Local knowledge store (recipes, facts, man pages)
//! - Man page caching (lazy, on-demand)
//! - Command --help output caching
//!
//! Key function: `search_docs()` for retrieval-augmented answering.

use crate::knowledge::{
    search_builtin_pack, KnowledgeDoc, KnowledgeSource, KnowledgeStore, KnowledgeStoreTrait,
    Provenance, RetrievalHit, RetrievalQuery,
};

/// Result from doc brain search
#[derive(Debug, Clone)]
pub struct DocSearchResult {
    /// Title of the document
    pub title: String,
    /// Relevant snippet
    pub snippet: String,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Source type
    pub source: KnowledgeSource,
    /// Document ID for reference
    pub doc_id: String,
}

impl From<RetrievalHit> for DocSearchResult {
    fn from(hit: RetrievalHit) -> Self {
        Self {
            title: hit.title,
            snippet: hit.snippet,
            confidence: hit.confidence,
            source: hit.source,
            doc_id: hit.doc_id,
        }
    }
}

/// Search documents across all sources
///
/// Priority:
/// 1. Built-in pack (fast, curated, high confidence)
/// 2. Knowledge store (recipes, facts, cached docs)
///
/// Returns up to `limit` results, sorted by relevance.
pub fn search_docs(query: &str, limit: usize) -> Vec<DocSearchResult> {
    let mut results = Vec::new();
    let half_limit = limit / 2 + 1;

    // 1. Search built-in pack first (fast, no I/O)
    for (score, entry) in search_builtin_pack(query, half_limit) {
        results.push(DocSearchResult {
            title: entry.title.to_string(),
            snippet: truncate_snippet(entry.body, 150),
            confidence: score_to_confidence(score),
            source: KnowledgeSource::BuiltIn,
            doc_id: format!("builtin:{}", entry.id),
        });
    }

    // 2. Search knowledge store if available
    if let Some(store_results) = search_knowledge_store(query, half_limit) {
        for hit in store_results {
            results.push(hit.into());
        }
    }

    // Sort by confidence descending
    results.sort_by(|a, b| b.confidence.cmp(&a.confidence));
    results.truncate(limit);
    results
}

/// Search for man page content
///
/// Searches the knowledge store for cached man pages.
/// If not found, returns None (caller can decide to fetch and cache).
pub fn search_man_page(command: &str) -> Option<DocSearchResult> {
    let store = KnowledgeStore::load_or_default();
    let query = RetrievalQuery::new(command)
        .with_sources(vec![KnowledgeSource::ManPage])
        .with_limit(1);

    store.query(&query).first().map(|hit| hit.clone().into())
}

/// Search for --help output
///
/// Searches the knowledge store for cached help output.
pub fn search_help_output(command: &str) -> Option<DocSearchResult> {
    let store = KnowledgeStore::load_or_default();
    let query = RetrievalQuery::new(command)
        .with_sources(vec![KnowledgeSource::HelpOutput])
        .with_limit(1);

    store.query(&query).first().map(|hit| hit.clone().into())
}

/// Cache a man page in the knowledge store
pub fn cache_man_page(command: &str, content: &str) -> bool {
    let mut store = KnowledgeStore::load_or_default();
    let doc = KnowledgeDoc::new(
        KnowledgeSource::ManPage,
        format!("man {}", command),
        content.to_string(),
        vec![command.to_string(), "man".to_string()],
        Provenance::from_command("doc_brain", &format!("man {}", command), 90),
    );
    store.upsert(doc).is_ok() && store.save().is_ok()
}

/// Cache a --help output in the knowledge store
pub fn cache_help_output(command: &str, content: &str) -> bool {
    let mut store = KnowledgeStore::load_or_default();
    let doc = KnowledgeDoc::new(
        KnowledgeSource::HelpOutput,
        format!("{} --help", command),
        content.to_string(),
        vec![command.to_string(), "help".to_string()],
        Provenance::from_command("doc_brain", &format!("{} --help", command), 85),
    );
    store.upsert(doc).is_ok() && store.save().is_ok()
}

/// Search the knowledge store (internal)
fn search_knowledge_store(query: &str, limit: usize) -> Option<Vec<RetrievalHit>> {
    let store = KnowledgeStore::load_or_default();
    let retrieval_query = RetrievalQuery::new(query).with_limit(limit);

    let results = store.query(&retrieval_query);
    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

/// Truncate snippet to reasonable length
fn truncate_snippet(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}

/// Convert search score to confidence (0-100)
fn score_to_confidence(score: u32) -> u8 {
    // Map score 0-50+ to confidence 50-95
    let base = 50u8;
    let bonus = (score as u8).min(45);
    base.saturating_add(bonus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_docs_builtin() {
        let results = search_docs("update arch linux", 5);
        assert!(!results.is_empty());
        assert!(results[0].title.to_lowercase().contains("update"));
        assert_eq!(results[0].source, KnowledgeSource::BuiltIn);
    }

    #[test]
    fn test_search_docs_limit() {
        let results = search_docs("pacman", 3);
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_score_to_confidence() {
        assert_eq!(score_to_confidence(0), 50);
        assert_eq!(score_to_confidence(30), 80);
        assert_eq!(score_to_confidence(50), 95);
        assert_eq!(score_to_confidence(100), 95); // Capped
    }

    #[test]
    fn test_truncate_snippet() {
        let short = "Hello world";
        assert_eq!(truncate_snippet(short, 50), short);

        let long = "This is a very long text that should be truncated at some point";
        let result = truncate_snippet(long, 20);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 23); // 20 + "..."
    }
}
