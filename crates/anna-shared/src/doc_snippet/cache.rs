//! Documentation cache for frequently used docs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::warn;

use super::types::DocSnippet;
use super::utils::{current_secs, extract_doc_keywords};

/// Documentation cache for frequently used docs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DocCache {
    /// Cached snippets by ID
    pub snippets: HashMap<String, DocSnippet>,
    /// Index: keyword -> snippet IDs
    pub keyword_index: HashMap<String, Vec<String>>,
    /// Cache metadata
    pub last_cleanup: u64,
}

impl DocCache {
    /// Load from disk
    pub fn load() -> Self {
        let path = Self::cache_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(cache) => return cache,
                    Err(e) => warn!("Failed to parse doc cache: {}", e),
                },
                Err(e) => warn!("Failed to read doc cache: {}", e),
            }
        }
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::cache_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        Ok(())
    }

    fn cache_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".anna")
            .join("doc_cache.json")
    }

    /// Add or update a snippet
    pub fn add(&mut self, snippet: DocSnippet) {
        // Index by keywords from excerpt
        let keywords = extract_doc_keywords(&snippet.excerpt);
        for kw in keywords {
            self.keyword_index
                .entry(kw)
                .or_default()
                .push(snippet.id.clone());
        }
        self.snippets.insert(snippet.id.clone(), snippet);
    }

    /// Find relevant snippets for a topic
    pub fn find(&self, topic: &str) -> Vec<&DocSnippet> {
        let keywords = extract_doc_keywords(topic);
        let mut scores: HashMap<&str, usize> = HashMap::new();

        for kw in &keywords {
            if let Some(ids) = self.keyword_index.get(kw) {
                for id in ids {
                    *scores.entry(id.as_str()).or_default() += 1;
                }
            }
        }

        let mut results: Vec<_> = scores
            .into_iter()
            .filter_map(|(id, score)| self.snippets.get(id).map(|s| (s, score)))
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(s, _)| s).take(5).collect()
    }

    /// Get snippet by ID
    pub fn get(&self, id: &str) -> Option<&DocSnippet> {
        self.snippets.get(id)
    }

    /// Cleanup old entries
    pub fn cleanup(&mut self, max_age_days: u64) {
        let threshold = current_secs().saturating_sub(max_age_days * 24 * 3600);
        let to_remove: Vec<_> = self
            .snippets
            .iter()
            .filter(|(_, s)| s.retrieved_at < threshold)
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.snippets.remove(&id);
        }

        // Rebuild keyword index
        self.keyword_index.clear();
        for (id, snippet) in &self.snippets {
            let keywords = extract_doc_keywords(&snippet.excerpt);
            for kw in keywords {
                self.keyword_index.entry(kw).or_default().push(id.clone());
            }
        }

        self.last_cleanup = current_secs();
    }
}
