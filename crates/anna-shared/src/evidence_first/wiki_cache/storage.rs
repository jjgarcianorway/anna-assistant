//! Wiki cache storage - manages cached wiki pages on disk.

use super::error::CacheError;
use super::types::{
    sanitize_filename, timestamp_now, CacheStats, WikiIndex, WikiPage, WikiSearchHit,
    WikiSearchResult,
};
use crate::evidence_first::citations::{Citation, CitationStore, EvidenceId};
use crate::evidence_first::sources::KnowledgeSource;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The wiki cache.
#[derive(Debug, Clone, Default)]
pub struct WikiCache {
    /// Cache directory.
    cache_dir: PathBuf,
    /// Index of pages by title.
    index: HashMap<String, PathBuf>,
    /// When index was last updated.
    index_updated: u64,
}

impl WikiCache {
    /// Create a new wiki cache.
    pub fn new(cache_dir: &Path) -> Self {
        let mut cache = Self {
            cache_dir: cache_dir.to_path_buf(),
            index: HashMap::new(),
            index_updated: 0,
        };
        cache.load_index();
        cache
    }

    /// Create with default directory.
    pub fn default_location() -> Self {
        Self::new(Path::new(super::WIKI_CACHE_DIR))
    }

    /// Load index from cache directory.
    fn load_index(&mut self) {
        let index_path = self.cache_dir.join("index.json");
        if let Ok(content) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<WikiIndex>(&content) {
                self.index = index
                    .pages
                    .into_iter()
                    .map(|(k, v)| (k, PathBuf::from(v)))
                    .collect();
                self.index_updated = index.updated_at;
            }
        }
    }

    /// Save index to cache directory.
    fn save_index(&self) -> Result<(), CacheError> {
        fs::create_dir_all(&self.cache_dir).map_err(|e| CacheError::IoError(e.to_string()))?;

        let index = WikiIndex {
            pages: self
                .index
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string_lossy().to_string()))
                .collect(),
            updated_at: self.index_updated,
        };

        let content = serde_json::to_string_pretty(&index)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;

        fs::write(self.cache_dir.join("index.json"), content)
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Get a cached page.
    pub fn get_page(&self, title: &str) -> Option<WikiPage> {
        let path = self.index.get(title)?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Add a page to cache.
    pub fn add_page(&mut self, page: &WikiPage) -> Result<(), CacheError> {
        fs::create_dir_all(&self.cache_dir).map_err(|e| CacheError::IoError(e.to_string()))?;

        let filename = sanitize_filename(&page.title);
        let path = self.cache_dir.join(format!("{}.json", filename));

        let content = serde_json::to_string_pretty(page)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;

        fs::write(&path, content).map_err(|e| CacheError::IoError(e.to_string()))?;

        self.index.insert(page.title.clone(), path);
        self.index_updated = timestamp_now();
        self.save_index()?;

        Ok(())
    }

    /// Search all cached pages.
    pub fn search(&self, query: &str) -> WikiSearchResult {
        let mut all_hits = Vec::new();
        let mut pages_searched = 0;

        for title in self.index.keys() {
            if let Some(page) = self.get_page(title) {
                pages_searched += 1;
                let hits = page.search(query);
                all_hits.extend(hits);

                if all_hits.len() >= 10 {
                    break;
                }
            }
        }

        // Sort by relevance (title match first)
        all_hits.sort_by(|a, b| {
            let a_title_match = a.page_title.to_lowercase().contains(&query.to_lowercase());
            let b_title_match = b.page_title.to_lowercase().contains(&query.to_lowercase());
            b_title_match.cmp(&a_title_match)
        });

        all_hits.truncate(10);

        WikiSearchResult {
            query: query.to_string(),
            hits: all_hits,
            pages_searched,
        }
    }

    /// Search and add citations to store.
    pub fn search_with_citations(
        &self,
        query: &str,
        store: &mut CitationStore,
    ) -> WikiSearchResult {
        let result = self.search(query);

        for hit in &result.hits {
            let evidence_id = EvidenceId::wiki(&hit.page_title);

            // Add raw evidence if not already present
            if !store.has_evidence(&evidence_id) {
                if let Some(page) = self.get_page(&hit.page_title) {
                    store.add_evidence(
                        evidence_id.clone(),
                        KnowledgeSource::ArchWiki(page.title.clone()),
                        &page.content,
                    );
                }
            }

            // Add citation
            store.add_citation(
                Citation::new(
                    evidence_id,
                    &format!("Arch Wiki: {}", hit.page_title),
                    &hit.excerpt,
                )
                .with_context(&hit.section_title),
            );
        }

        result
    }

    /// List all cached pages.
    pub fn list_pages(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }

    /// Get cache stats.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            page_count: self.index.len(),
            last_updated: self.index_updated,
            cache_dir: self.cache_dir.to_string_lossy().to_string(),
        }
    }

    /// Check if cache is stale (older than 30 days).
    pub fn is_stale(&self) -> bool {
        let now = timestamp_now();
        let thirty_days = 30 * 24 * 60 * 60;
        now - self.index_updated > thirty_days
    }

    /// Clear the cache.
    pub fn clear(&mut self) -> Result<(), CacheError> {
        for path in self.index.values() {
            let _ = fs::remove_file(path);
        }
        self.index.clear();
        self.index_updated = 0;
        self.save_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_cache_dir() -> PathBuf {
        env::temp_dir().join(format!("anna_wiki_test_{}", timestamp_now()))
    }

    #[test]
    fn test_wiki_cache_new() {
        let cache_dir = temp_cache_dir();
        let cache = WikiCache::new(&cache_dir);

        // New cache should be empty
        assert!(cache.list_pages().is_empty());
        assert_eq!(cache.stats().page_count, 0);
    }

    #[test]
    fn test_wiki_cache_stale_check() {
        let cache_dir = temp_cache_dir();
        let cache = WikiCache::new(&cache_dir);

        // New cache with no updates is stale (index_updated is 0)
        assert!(cache.is_stale());
    }
}
