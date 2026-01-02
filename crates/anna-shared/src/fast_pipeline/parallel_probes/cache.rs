//! Parallel Probe Cache - v0.0.438.
//!
//! Smart caching for probe results with TTL.

use super::types::CACHE_TTL_SECONDS;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cached probe entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cached value.
    pub value: String,
    /// When cached.
    pub cached_at: Instant,
    /// TTL for this entry.
    pub ttl: Duration,
}

impl CacheEntry {
    /// Create new cache entry.
    pub fn new(value: &str, ttl: Duration) -> Self {
        Self {
            value: value.to_string(),
            cached_at: Instant::now(),
            ttl,
        }
    }

    /// Check if expired.
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    /// Get remaining TTL.
    pub fn remaining_ttl(&self) -> Duration {
        let elapsed = self.cached_at.elapsed();
        if elapsed >= self.ttl {
            Duration::ZERO
        } else {
            self.ttl - elapsed
        }
    }
}

/// Probe cache.
#[derive(Debug, Default)]
pub struct ProbeCache {
    /// Cached entries by probe ID.
    entries: HashMap<String, CacheEntry>,
    /// Default TTL.
    default_ttl: Duration,
    /// Hit count.
    hits: usize,
    /// Miss count.
    misses: usize,
}

impl ProbeCache {
    /// Create new cache with default TTL.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl: Duration::from_secs(CACHE_TTL_SECONDS),
            hits: 0,
            misses: 0,
        }
    }

    /// Create with custom TTL.
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl: Duration::from_secs(ttl_seconds),
            hits: 0,
            misses: 0,
        }
    }

    /// Get cached value if not expired.
    pub fn get(&mut self, probe_id: &str) -> Option<String> {
        if let Some(entry) = self.entries.get(probe_id) {
            if !entry.is_expired() {
                self.hits += 1;
                return Some(entry.value.clone());
            }
            // Expired - will be replaced
        }
        self.misses += 1;
        None
    }

    /// Set cached value.
    pub fn set(&mut self, probe_id: &str, value: &str) {
        self.entries.insert(
            probe_id.to_string(),
            CacheEntry::new(value, self.default_ttl),
        );
    }

    /// Set with custom TTL.
    pub fn set_with_ttl(&mut self, probe_id: &str, value: &str, ttl: Duration) {
        self.entries
            .insert(probe_id.to_string(), CacheEntry::new(value, ttl));
    }

    /// Clear expired entries.
    pub fn cleanup(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    /// Get cache stats.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.entries.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries.
    pub size: usize,
    /// Cache hits.
    pub hits: usize,
    /// Cache misses.
    pub misses: usize,
    /// Hit rate (0.0-1.0).
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_cache() {
        let mut cache = ProbeCache::new();

        // Miss
        assert!(cache.get("sys.mem").is_none());
        assert_eq!(cache.stats().misses, 1);

        // Set and hit
        cache.set("sys.mem", "4 GB");
        assert_eq!(cache.get("sys.mem"), Some("4 GB".to_string()));
        assert_eq!(cache.stats().hits, 1);
    }
}
