//! Index search operations (v0.0.429).

use super::types::DocIndex;
use crate::doc_engine::{DocSnippet, DocSourceKind};

impl DocIndex {
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

    /// Calculate relevance score for a snippet
    pub(super) fn score_match(&self, snippet: &DocSnippet, query_words: &[String]) -> u8 {
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
