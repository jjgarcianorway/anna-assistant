//! Idempotency store for deduplicating submit requests (Phase 1.11)

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Idempotency key entry
#[derive(Debug, Clone)]
struct IdempotencyEntry {
    key: String,
    inserted_at: Instant,
}

/// LRU-based idempotency store with TTL
pub struct IdempotencyStore {
    cache: Arc<Mutex<LruCache<String, IdempotencyEntry>>>,
    ttl: Duration,
}

impl IdempotencyStore {
    /// Create new idempotency store
    ///
    /// * `capacity` - Maximum number of keys to track
    /// * `ttl` - Time-to-live for keys (default: 10 minutes)
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap());

        Self {
            cache: Arc::new(Mutex::new(cache)),
            ttl,
        }
    }

    /// Check if key exists and is not expired, insert if new
    ///
    /// Returns:
    /// * `Ok(false)` - New key, inserted
    /// * `Ok(true)` - Duplicate key within TTL window
    pub async fn check_and_insert(&self, key: &str) -> bool {
        let mut cache = self.cache.lock().await;
        let now = Instant::now();

        // Check if key exists
        if let Some(entry) = cache.get(key) {
            // Check if expired
            if now.duration_since(entry.inserted_at) < self.ttl {
                // Duplicate within TTL
                return true;
            }
            // Expired, remove and fall through to insert
            cache.pop(key);
        }

        // Insert new key
        cache.put(
            key.to_string(),
            IdempotencyEntry {
                key: key.to_string(),
                inserted_at: now,
            },
        );

        false
    }

    /// Prune expired entries (called periodically)
    pub async fn prune_expired(&self) {
        let mut cache = self.cache.lock().await;
        let now = Instant::now();

        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.inserted_at) >= self.ttl)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            cache.pop(&key);
        }
    }

    /// Get current cache size
    pub async fn len(&self) -> usize {
        self.cache.lock().await.len()
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        // Default: 10,000 keys, 10 minute TTL
        Self::new(10_000, Duration::from_secs(600))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_idempotency_new_key() {
        let store = IdempotencyStore::new(100, Duration::from_secs(60));

        let is_duplicate = store.check_and_insert("key1").await;
        assert!(!is_duplicate, "First insert should not be a duplicate");
    }

    #[tokio::test]
    async fn test_idempotency_duplicate_key() {
        let store = IdempotencyStore::new(100, Duration::from_secs(60));

        store.check_and_insert("key1").await;
        let is_duplicate = store.check_and_insert("key1").await;
        assert!(is_duplicate, "Second insert should be a duplicate");
    }

    #[tokio::test]
    async fn test_idempotency_expiration() {
        let store = IdempotencyStore::new(100, Duration::from_millis(100));

        store.check_and_insert("key1").await;
        tokio::time::sleep(Duration::from_millis(150)).await;

        let is_duplicate = store.check_and_insert("key1").await;
        assert!(!is_duplicate, "Key should have expired");
    }

    #[tokio::test]
    async fn test_idempotency_lru_eviction() {
        let store = IdempotencyStore::new(2, Duration::from_secs(60));

        store.check_and_insert("key1").await;
        store.check_and_insert("key2").await;
        store.check_and_insert("key3").await; // Should evict key1

        let is_dup_key2 = store.check_and_insert("key2").await;
        let is_dup_key3 = store.check_and_insert("key3").await;

        assert!(is_dup_key2, "key2 should still be in cache");
        assert!(is_dup_key3, "key3 should still be in cache");
    }
}
