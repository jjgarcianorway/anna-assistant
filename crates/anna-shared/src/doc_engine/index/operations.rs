//! Index CRUD operations (v0.0.429).

use super::types::DocIndex;
use crate::doc_engine::{DocSnippet, DocSourceKind, MAX_SNIPPET_SIZE};

impl DocIndex {
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
}
