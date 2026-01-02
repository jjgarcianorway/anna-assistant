// v0.0.586: Settings Cache Implementation (Phase 162)
// Main cache implementation for settings values

use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

use super::types::{CacheEntry, CacheStats, EvictionPolicy};

/// Settings cache
#[derive(Debug, Clone, Default)]
pub struct SettingsCache {
    /// Cache entries
    entries: HashMap<String, CacheEntry>,
    /// Eviction policy
    policy: EvictionPolicy,
    /// Max entries
    max_entries: usize,
    /// Max size in bytes
    max_size: usize,
    /// Default TTL in seconds
    default_ttl: Option<i64>,
    /// Statistics
    stats: CacheStats,
}

impl SettingsCache {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            max_entries: 1000,
            max_size: 10 * 1024 * 1024, // 10MB
            ..Default::default()
        }
    }

    /// Set eviction policy
    pub fn policy(mut self, policy: EvictionPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set max entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set default TTL
    pub fn default_ttl(mut self, seconds: i64) -> Self {
        self.default_ttl = Some(seconds);
        self
    }

    /// Get value from cache
    pub fn get(&mut self, key: &str) -> Option<&str> {
        if let Some(entry) = self.entries.get_mut(key) {
            if entry.is_expired() {
                self.stats.misses += 1;
                return None;
            }
            entry.touch();
            self.stats.hits += 1;
            Some(&entry.value)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Put value in cache
    pub fn put(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key_str = key.into();
        let mut entry = CacheEntry::new(key_str.clone(), value);

        if let Some(ttl) = self.default_ttl {
            entry = entry.ttl(ttl);
        }

        self.stats.size += entry.size;
        self.entries.insert(key_str, entry);
        self.stats.entries = self.entries.len();

        self.evict_if_needed();
    }

    /// Put with category
    pub fn put_with_category(&mut self, key: impl Into<String>, value: impl Into<String>, cat: SettingsCategory) {
        let key_str = key.into();
        let mut entry = CacheEntry::new(key_str.clone(), value).category(cat);

        if let Some(ttl) = self.default_ttl {
            entry = entry.ttl(ttl);
        }

        self.stats.size += entry.size;
        self.entries.insert(key_str, entry);
        self.stats.entries = self.entries.len();

        self.evict_if_needed();
    }

    /// Remove entry
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(entry) = self.entries.remove(key) {
            self.stats.size = self.stats.size.saturating_sub(entry.size);
            self.stats.entries = self.entries.len();
            true
        } else {
            false
        }
    }

    /// Invalidate by category
    pub fn invalidate_category(&mut self, category: SettingsCategory) {
        let keys: Vec<_> = self.entries
            .iter()
            .filter(|(_, e)| e.category == Some(category))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys {
            self.remove(&key);
        }
    }

    /// Mark category as stale
    pub fn mark_stale_category(&mut self, category: SettingsCategory) {
        for entry in self.entries.values_mut() {
            if entry.category == Some(category) {
                entry.mark_stale();
            }
        }
    }

    /// Clear expired entries
    pub fn clear_expired(&mut self) {
        let expired: Vec<_> = self.entries
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired {
            self.remove(&key);
        }
    }

    /// Evict entries if needed
    fn evict_if_needed(&mut self) {
        while self.entries.len() > self.max_entries || self.stats.size > self.max_size {
            if let Some(key) = self.select_eviction() {
                self.remove(&key);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }
    }

    /// Select entry to evict
    fn select_eviction(&self) -> Option<String> {
        match self.policy {
            EvictionPolicy::LRU => {
                self.entries.iter()
                    .min_by_key(|(_, e)| e.last_access)
                    .map(|(k, _)| k.clone())
            }
            EvictionPolicy::LFU => {
                self.entries.iter()
                    .min_by_key(|(_, e)| e.access_count)
                    .map(|(k, _)| k.clone())
            }
            EvictionPolicy::FIFO => {
                self.entries.iter()
                    .min_by_key(|(_, e)| e.created_at)
                    .map(|(k, _)| k.clone())
            }
            EvictionPolicy::TTL => {
                self.entries.iter()
                    .filter(|(_, e)| e.expires_at.is_some())
                    .min_by_key(|(_, e)| e.expires_at)
                    .map(|(k, _)| k.clone())
            }
        }
    }

    /// Contains key
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.entries = 0;
        self.stats.size = 0;
    }
}
