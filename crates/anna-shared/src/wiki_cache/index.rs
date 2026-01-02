//! Wiki cache index and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::entry::WikiCacheEntry;
use super::utils::normalize_title;

/// Wiki cache index and management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WikiCacheIndex {
    /// Cached pages by normalized title
    pub entries: HashMap<String, WikiCacheEntry>,
    /// Cache version for migrations
    pub version: u32,
    /// Maximum cache age in days
    pub max_age_days: u64,
}

impl WikiCacheIndex {
    /// Create a new cache index
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: 1,
            max_age_days: 30, // Default: 30 days
        }
    }

    /// Get default cache path
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("anna")
            .join("wiki")
    }

    /// Get index file path
    pub fn index_path() -> PathBuf {
        Self::cache_dir().join("index.json")
    }

    /// Load cache index from disk
    pub fn load() -> Self {
        let path = Self::index_path();
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::new(),
        }
    }

    /// Save cache index to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::index_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Add or update a cache entry
    pub fn upsert(&mut self, entry: WikiCacheEntry) {
        self.entries.insert(normalize_title(&entry.title), entry);
    }

    /// Get a cache entry by title
    pub fn get(&self, title: &str) -> Option<&WikiCacheEntry> {
        self.entries.get(&normalize_title(title))
    }

    /// Get a mutable cache entry by title
    pub fn get_mut(&mut self, title: &str) -> Option<&mut WikiCacheEntry> {
        self.entries.get_mut(&normalize_title(title))
    }

    /// Record access to a page
    pub fn record_access(&mut self, title: &str) {
        if let Some(entry) = self.get_mut(title) {
            entry.last_accessed = super::utils::now_timestamp();
        }
    }

    /// Check if page is cached
    pub fn has(&self, title: &str) -> bool {
        self.entries.contains_key(&normalize_title(title))
    }

    /// Get stale entries that need refresh
    pub fn stale_entries(&self) -> Vec<&WikiCacheEntry> {
        self.entries
            .values()
            .filter(|e| e.is_stale(self.max_age_days))
            .collect()
    }

    /// Get least recently accessed entries
    pub fn least_accessed(&self, limit: usize) -> Vec<&WikiCacheEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by_key(|e| e.last_accessed);
        entries.into_iter().take(limit).collect()
    }

    /// Total cache size in bytes
    pub fn total_size(&self) -> usize {
        self.entries.values().map(|e| e.size_bytes).sum()
    }

    /// Number of cached pages
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Remove an entry
    pub fn remove(&mut self, title: &str) -> Option<WikiCacheEntry> {
        self.entries.remove(&normalize_title(title))
    }

    /// Remove stale entries
    pub fn prune_stale(&mut self) -> usize {
        let stale_keys: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, e)| e.is_stale(self.max_age_days))
            .map(|(k, _)| k.clone())
            .collect();

        let count = stale_keys.len();
        for key in stale_keys {
            self.entries.remove(&key);
        }
        count
    }

    /// Remove least accessed entries to meet size limit
    pub fn prune_to_size(&mut self, max_bytes: usize) -> usize {
        let mut removed = 0;
        while self.total_size() > max_bytes && !self.entries.is_empty() {
            // Find least recently accessed
            if let Some(key) = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&key);
                removed += 1;
            } else {
                break;
            }
        }
        removed
    }
}

/// Essential pages that should be cached
pub fn essential_pages() -> Vec<&'static str> {
    crate::doc_engine::wiki_reader::get_essential_wiki_pages()
}

/// Pages that are missing from cache
pub fn missing_essential(index: &WikiCacheIndex) -> Vec<&'static str> {
    essential_pages()
        .into_iter()
        .filter(|p| !index.has(p))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::utils::now_timestamp;

    #[test]
    fn test_cache_index_operations() {
        let mut index = WikiCacheIndex::new();

        let entry = WikiCacheEntry::new("systemd", "http://wiki/systemd", 5000, "hash1");
        index.upsert(entry);

        assert!(index.has("systemd"));
        assert!(index.has("SYSTEMD")); // Case insensitive
        assert!(!index.has("nonexistent"));

        assert_eq!(index.count(), 1);
        assert_eq!(index.total_size(), 5000);
    }

    #[test]
    fn test_missing_essential() {
        let index = WikiCacheIndex::new();
        let missing = missing_essential(&index);
        // Should be many missing since cache is empty
        assert!(!missing.is_empty());
        assert!(missing.contains(&"systemd"));
    }

    #[test]
    fn test_prune_stale() {
        let mut index = WikiCacheIndex::new();
        index.max_age_days = 30;

        // Add fresh entry
        let fresh = WikiCacheEntry::new("fresh", "http://fresh", 100, "h1");
        index.upsert(fresh);

        // Add stale entry
        let mut stale = WikiCacheEntry::new("stale", "http://stale", 100, "h2");
        stale.cached_at = now_timestamp() - (31 * 86400);
        index.upsert(stale);

        assert_eq!(index.count(), 2);
        let pruned = index.prune_stale();
        assert_eq!(pruned, 1);
        assert_eq!(index.count(), 1);
        assert!(index.has("fresh"));
        assert!(!index.has("stale"));
    }
}
