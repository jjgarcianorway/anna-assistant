//! Knowledge fetcher result types.

use crate::knowledge_v2::snippet::{KnowledgeSnippet, KnowledgeSource};

/// Result of a knowledge fetch operation
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Fetched snippets
    pub snippets: Vec<KnowledgeSnippet>,
    /// Topics that were searched
    pub topics_searched: Vec<String>,
    /// Sources that were checked
    pub sources_checked: Vec<KnowledgeSource>,
    /// Whether any knowledge was found
    pub has_knowledge: bool,
    /// Fetch duration in milliseconds
    pub duration_ms: u64,
}

impl FetchResult {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            snippets: vec![],
            topics_searched: vec![],
            sources_checked: vec![],
            has_knowledge: false,
            duration_ms: 0,
        }
    }

    /// Get primary citation if available
    pub fn primary_citation(&self) -> Option<String> {
        self.snippets.first().map(|s| s.primary_citation())
    }

    /// Get all citations
    pub fn all_citations(&self) -> Vec<String> {
        self.snippets
            .iter()
            .flat_map(|s| s.citations.clone())
            .collect()
    }
}
