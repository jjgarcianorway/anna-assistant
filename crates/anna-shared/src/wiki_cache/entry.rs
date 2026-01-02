//! Wiki cache entry metadata and operations.

use serde::{Deserialize, Serialize};

use super::utils::now_timestamp;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_stale() {
        let mut entry = WikiCacheEntry::new("test", "http://test", 100, "abc");
        // Not stale when just created
        assert!(!entry.is_stale(30));

        // Force old timestamp
        entry.cached_at = now_timestamp() - (31 * 86400);
        assert!(entry.is_stale(30));
    }
}
