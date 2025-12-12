//! Arch Wiki Local Cache (v0.0.472).
//!
//! Local caching of Arch Wiki pages for offline knowledge.
//! Per VISION.md: "Store local copies of wiki pages if linked from Arch Wiki"
//!
//! Features:
//! - Download and cache essential wiki pages
//! - Track cache freshness and staleness
//! - Manage cache size and cleanup

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Wiki cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiCacheEntry {
    /// Page title (normalized)
    pub title: String,
    /// URL of the page
    pub url: String,
    /// When the page was cached
    pub cached_at: u64,
    /// Last time the page was accessed
    pub last_accessed: u64,
    /// Size in bytes
    pub size_bytes: usize,
    /// Content hash for change detection
    pub content_hash: String,
}

impl WikiCacheEntry {
    /// Create a new cache entry
    pub fn new(title: &str, url: &str, size: usize, hash: &str) -> Self {
        let now = now_timestamp();
        Self {
            title: title.to_string(),
            url: url.to_string(),
            cached_at: now,
            last_accessed: now,
            size_bytes: size,
            content_hash: hash.to_string(),
        }
    }

    /// Check if cache entry is stale (older than max_age_days)
    pub fn is_stale(&self, max_age_days: u64) -> bool {
        let now = now_timestamp();
        let age_secs = now.saturating_sub(self.cached_at);
        age_secs > (max_age_days * 86400)
    }

    /// Age in days
    pub fn age_days(&self) -> u64 {
        let now = now_timestamp();
        now.saturating_sub(self.cached_at) / 86400
    }
}

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
            entry.last_accessed = now_timestamp();
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

/// Get path to cached wiki page
pub fn get_cache_path(title: &str) -> PathBuf {
    let normalized = normalize_title(title);
    WikiCacheIndex::cache_dir().join(format!("{}.txt", normalized))
}

/// Read cached wiki page content
pub fn read_cached(title: &str) -> Option<String> {
    let path = get_cache_path(title);
    fs::read_to_string(path).ok()
}

/// Write wiki page to cache
pub fn write_cached(title: &str, content: &str) -> Result<WikiCacheEntry, std::io::Error> {
    let cache_dir = WikiCacheIndex::cache_dir();
    fs::create_dir_all(&cache_dir)?;

    let path = get_cache_path(title);
    fs::write(&path, content)?;

    let hash = simple_hash(content);
    let entry = WikiCacheEntry::new(
        title,
        &format!("https://wiki.archlinux.org/title/{}", title.replace(' ', "_")),
        content.len(),
        &hash,
    );

    Ok(entry)
}

/// Delete cached wiki page
pub fn delete_cached(title: &str) -> bool {
    let path = get_cache_path(title);
    fs::remove_file(path).is_ok()
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached pages
    pub page_count: usize,
    /// Total size in bytes
    pub total_bytes: usize,
    /// Number of stale pages
    pub stale_count: usize,
    /// Oldest page age in days
    pub oldest_days: u64,
    /// Average page age in days
    pub avg_age_days: u64,
}

impl CacheStats {
    /// Format as display string
    pub fn display(&self) -> String {
        let size_str = if self.total_bytes > 1024 * 1024 {
            format!("{:.1}MB", self.total_bytes as f64 / (1024.0 * 1024.0))
        } else if self.total_bytes > 1024 {
            format!("{:.1}KB", self.total_bytes as f64 / 1024.0)
        } else {
            format!("{}B", self.total_bytes)
        };

        format!(
            "pages: {}, size: {}, stale: {}, oldest: {}d, avg: {}d",
            self.page_count, size_str, self.stale_count, self.oldest_days, self.avg_age_days
        )
    }
}

/// Get cache statistics
pub fn get_cache_stats(index: &WikiCacheIndex) -> CacheStats {
    let now = now_timestamp();
    let mut oldest: u64 = 0;
    let mut total_age: u64 = 0;

    for entry in index.entries.values() {
        let age = now.saturating_sub(entry.cached_at);
        total_age += age;
        if age > oldest {
            oldest = age;
        }
    }

    let avg_age = if index.entries.is_empty() {
        0
    } else {
        total_age / index.entries.len() as u64
    };

    CacheStats {
        page_count: index.count(),
        total_bytes: index.total_size(),
        stale_count: index.stale_entries().len(),
        oldest_days: oldest / 86400,
        avg_age_days: avg_age / 86400,
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

/// Normalize title for consistent lookup
fn normalize_title(title: &str) -> String {
    title.to_lowercase().replace(' ', "_").replace('/', "_")
}

/// Simple string hash for change detection
fn simple_hash(s: &str) -> String {
    let mut hash: u64 = 0;
    for (i, b) in s.bytes().enumerate() {
        hash = hash.wrapping_add((b as u64).wrapping_mul((i + 1) as u64));
    }
    format!("{:016x}", hash)
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(normalize_title("Arch Wiki"), "arch_wiki");
        assert_eq!(normalize_title("Systemd/User"), "systemd_user");
        assert_eq!(normalize_title("GRUB"), "grub");
    }

    #[test]
    fn test_cache_entry_stale() {
        let mut entry = WikiCacheEntry::new("test", "http://test", 100, "abc");
        // Not stale when just created
        assert!(!entry.is_stale(30));

        // Force old timestamp
        entry.cached_at = now_timestamp() - (31 * 86400);
        assert!(entry.is_stale(30));
    }

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
    fn test_simple_hash() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("world");
        let h3 = simple_hash("hello");

        assert_ne!(h1, h2);
        assert_eq!(h1, h3);
    }

    #[test]
    fn test_cache_stats_display() {
        let stats = CacheStats {
            page_count: 50,
            total_bytes: 1024 * 1024 + 512 * 1024,
            stale_count: 5,
            oldest_days: 45,
            avg_age_days: 15,
        };
        let output = stats.display();
        assert!(output.contains("50"));
        assert!(output.contains("MB"));
        assert!(output.contains("stale"));
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
