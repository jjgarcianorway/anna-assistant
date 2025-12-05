//! Retrieval API for knowledge queries (v0.0.32R).
//!
//! Deterministic retrieval with source filtering and snippet extraction.

use super::sources::{KnowledgeDoc, KnowledgeSource};
use serde::{Deserialize, Serialize};

/// Query for knowledge retrieval
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetrievalQuery {
    /// Search text
    pub text: String,
    /// Filter by sources (empty = all sources)
    pub sources: Vec<KnowledgeSource>,
    /// Filter by any of these tags (empty = no tag filter)
    pub tags_any: Vec<String>,
    /// Maximum results to return
    pub limit: usize,
    /// Minimum confidence threshold (0-100)
    pub min_confidence: u8,
}

impl RetrievalQuery {
    /// Create a new query
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            sources: vec![],
            tags_any: vec![],
            limit: 10,
            min_confidence: 0,
        }
    }

    /// Filter by sources
    pub fn with_sources(mut self, sources: Vec<KnowledgeSource>) -> Self {
        self.sources = sources;
        self
    }

    /// Filter by tags (any match)
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags_any = tags;
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set minimum confidence
    pub fn with_min_confidence(mut self, confidence: u8) -> Self {
        self.min_confidence = confidence;
        self
    }
}

/// A retrieval hit with score and snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHit {
    /// Document ID
    pub doc_id: String,
    /// Relevance score (higher = better)
    pub score: i32,
    /// Extracted snippet around matched terms
    pub snippet: String,
    /// Document title
    pub title: String,
    /// Document source
    pub source: KnowledgeSource,
    /// Confidence from provenance
    pub confidence: u8,
}

impl RetrievalHit {
    /// Create a hit from a document and score
    pub fn from_doc(doc: &KnowledgeDoc, score: i32, query: &str) -> Self {
        let snippet = extract_snippet(&doc.body, query);
        Self {
            doc_id: doc.id.clone(),
            score,
            snippet,
            title: doc.title.clone(),
            source: doc.source.clone(),
            confidence: doc.provenance.confidence,
        }
    }
}

/// Extract a deterministic snippet around query terms
pub fn extract_snippet(body: &str, query: &str) -> String {
    const SNIPPET_LENGTH: usize = 150;
    const CONTEXT_CHARS: usize = 50;

    let body_lower = body.to_lowercase();
    let query_lower = query.to_lowercase();

    // Find first occurrence of any query term
    let query_terms: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|s| s.len() >= 2)
        .collect();

    let mut best_pos: Option<usize> = None;
    for term in &query_terms {
        if let Some(pos) = body_lower.find(term) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
            }
        }
    }

    match best_pos {
        Some(pos) => {
            // Start a bit before the match
            let start = pos.saturating_sub(CONTEXT_CHARS);
            // Find word boundary
            let start = body[..start]
                .rfind(|c: char| c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(start);

            let end = (start + SNIPPET_LENGTH).min(body.len());
            // Find word boundary
            let end = body[end..]
                .find(|c: char| c.is_whitespace())
                .map(|i| end + i)
                .unwrap_or(end);

            let mut snippet = body[start..end].to_string();

            // Add ellipsis if truncated
            if start > 0 {
                snippet = format!("...{}", snippet);
            }
            if end < body.len() {
                snippet = format!("{}...", snippet);
            }

            snippet
        }
        None => {
            // No match found, return start of body
            let end = SNIPPET_LENGTH.min(body.len());
            let end = body[end..]
                .find(|c: char| c.is_whitespace())
                .map(|i| end + i)
                .unwrap_or(end);

            if end < body.len() {
                format!("{}...", &body[..end])
            } else {
                body.to_string()
            }
        }
    }
}

/// Check if a document matches the query filters
pub fn doc_matches_filters(doc: &KnowledgeDoc, query: &RetrievalQuery) -> bool {
    // Check source filter
    if !query.sources.is_empty() && !query.sources.contains(&doc.source) {
        return false;
    }

    // Check tag filter (any match)
    if !query.tags_any.is_empty() {
        let has_matching_tag = doc.tags.iter().any(|t| query.tags_any.contains(t));
        if !has_matching_tag {
            return false;
        }
    }

    // Check confidence threshold
    if doc.provenance.confidence < query.min_confidence {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::sources::Provenance;

    fn test_doc(title: &str, body: &str, source: KnowledgeSource) -> KnowledgeDoc {
        KnowledgeDoc::new(
            source,
            title,
            body,
            vec![],
            Provenance::computed("test", 100),
        )
    }

    #[test]
    fn test_extract_snippet_with_match() {
        let body = "This is a long document about vim configuration. You can set various options in your vimrc file to customize the editor behavior.";
        let snippet = extract_snippet(body, "vimrc");
        assert!(snippet.contains("vimrc"));
        assert!(snippet.len() <= 200); // Reasonable length
    }

    #[test]
    fn test_extract_snippet_no_match() {
        let body = "This is a document about something else entirely.";
        let snippet = extract_snippet(body, "vim");
        // Should return start of body
        assert!(snippet.starts_with("This is"));
    }

    #[test]
    fn test_doc_matches_source_filter() {
        let doc = test_doc("Test", "body", KnowledgeSource::Recipe);
        let query = RetrievalQuery::new("test")
            .with_sources(vec![KnowledgeSource::Recipe]);
        assert!(doc_matches_filters(&doc, &query));

        let query2 = RetrievalQuery::new("test")
            .with_sources(vec![KnowledgeSource::ArchWiki]);
        assert!(!doc_matches_filters(&doc, &query2));
    }

    #[test]
    fn test_doc_matches_tag_filter() {
        let mut doc = test_doc("Test", "body", KnowledgeSource::Recipe);
        doc.tags = vec!["vim".to_string(), "editor".to_string()];

        let query = RetrievalQuery::new("test")
            .with_tags(vec!["vim".to_string()]);
        assert!(doc_matches_filters(&doc, &query));

        let query2 = RetrievalQuery::new("test")
            .with_tags(vec!["emacs".to_string()]);
        assert!(!doc_matches_filters(&doc, &query2));
    }

    #[test]
    fn test_doc_matches_confidence_filter() {
        let doc = test_doc("Test", "body", KnowledgeSource::Recipe);
        // doc has confidence 100

        let query = RetrievalQuery::new("test").with_min_confidence(80);
        assert!(doc_matches_filters(&doc, &query));

        let low_conf_doc = KnowledgeDoc::new(
            KnowledgeSource::Recipe,
            "Test",
            "body",
            vec![],
            Provenance::computed("test", 50),
        );
        assert!(!doc_matches_filters(&low_conf_doc, &query));
    }

    // Golden test for deterministic snippets
    #[test]
    fn golden_snippet_extraction() {
        let body = "The pacman package manager is the default on Arch Linux. Use pacman -S to install packages and pacman -R to remove them.";
        let snippet1 = extract_snippet(body, "pacman");
        let snippet2 = extract_snippet(body, "pacman");
        assert_eq!(snippet1, snippet2); // Deterministic
    }
}
