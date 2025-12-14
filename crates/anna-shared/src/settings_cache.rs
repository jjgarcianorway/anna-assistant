// v0.0.586: Settings Cache (Phase 162)
// Caching for settings values to improve performance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Cache entry state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CacheState {
    /// Valid cached value
    #[default]
    Valid,
    /// Stale (needs refresh)
    Stale,
    /// Expired (will be removed)
    Expired,
    /// Loading
    Loading,
}

impl std::fmt::Display for CacheState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Stale => write!(f, "stale"),
            Self::Expired => write!(f, "expired"),
            Self::Loading => write!(f, "loading"),
        }
    }
}

/// Cache eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least recently used
    #[default]
    LRU,
    /// Least frequently used
    LFU,
    /// First in first out
    FIFO,
    /// Time-based expiry
    TTL,
}

impl std::fmt::Display for EvictionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LRU => write!(f, "LRU"),
            Self::LFU => write!(f, "LFU"),
            Self::FIFO => write!(f, "FIFO"),
            Self::TTL => write!(f, "TTL"),
        }
    }
}

/// Cached value entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Entry key
    pub key: String,
    /// Cached value (serialized)
    pub value: String,
    /// Category
    pub category: Option<SettingsCategory>,
    /// State
    pub state: CacheState,
    /// Created time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed
    pub last_access: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Expiry time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Size in bytes
    pub size: usize,
}

impl CacheEntry {
    /// Create new cache entry
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        let value_str = value.into();
        let size = value_str.len();
        Self {
            key: key.into(),
            value: value_str,
            category: None,
            state: CacheState::Valid,
            created_at: now,
            last_access: now,
            access_count: 0,
            expires_at: None,
            size,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set TTL in seconds
    pub fn ttl(mut self, seconds: i64) -> Self {
        self.expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(seconds));
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }

    /// Mark as accessed
    pub fn touch(&mut self) {
        self.last_access = chrono::Utc::now();
        self.access_count += 1;
    }

    /// Mark as stale
    pub fn mark_stale(&mut self) {
        self.state = CacheState::Stale;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total hits
    pub hits: u64,
    /// Total misses
    pub misses: u64,
    /// Evictions
    pub evictions: u64,
    /// Current entry count
    pub entries: usize,
    /// Total size in bytes
    pub size: usize,
}

impl CacheStats {
    /// Hit rate (0.0-1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

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

/// Format cache status
pub fn format_cache(cache: &SettingsCache) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Cache ===\n\n");
    let stats = cache.stats();
    output.push_str(&format!(
        "Entries: {} | Size: {} bytes\n",
        stats.entries, stats.size
    ));
    output.push_str(&format!(
        "Hits: {} | Misses: {} | Rate: {:.1}%\n",
        stats.hits, stats.misses, stats.hit_rate() * 100.0
    ));
    output.push_str(&format!("Evictions: {}\n", stats.evictions));

    output
}

/// Check if query is about cache
pub fn is_cache_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("cache")
        || lower.contains("cached")
        || lower.contains("memory")
}

/// Fun fact about cache
pub fn settings_cache_fun_fact() -> &'static str {
    "Anna caches settings for lightning-fast access!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_state_display() {
        assert_eq!(format!("{}", CacheState::Valid), "valid");
        assert_eq!(format!("{}", CacheState::Stale), "stale");
    }

    #[test]
    fn test_eviction_policy_display() {
        assert_eq!(format!("{}", EvictionPolicy::LRU), "LRU");
        assert_eq!(format!("{}", EvictionPolicy::FIFO), "FIFO");
    }

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry::new("key", "value");
        assert_eq!(entry.key, "key");
        assert_eq!(entry.value, "value");
        assert_eq!(entry.state, CacheState::Valid);
    }

    #[test]
    fn test_cache_entry_touch() {
        let mut entry = CacheEntry::new("key", "value");
        assert_eq!(entry.access_count, 0);
        entry.touch();
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
        stats.hits = 7;
        stats.misses = 3;
        assert!((stats.hit_rate() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_cache_new() {
        let cache = SettingsCache::new();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_put_get() {
        let mut cache = SettingsCache::new();
        cache.put("key1", "value1");
        assert_eq!(cache.get("key1"), Some("value1"));
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = SettingsCache::new();
        cache.put("key1", "value1");
        assert!(cache.remove("key1"));
        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = SettingsCache::new();
        cache.put("key1", "value1");
        let _ = cache.get("key1");
        let _ = cache.get("key2");
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_format_cache() {
        let cache = SettingsCache::new();
        let output = format_cache(&cache);
        assert!(output.contains("Cache"));
    }

    #[test]
    fn test_is_cache_query() {
        assert!(is_cache_query("clear cache"));
        assert!(is_cache_query("cached settings"));
        assert!(!is_cache_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_cache_fun_fact();
        assert!(fact.contains("cache"));
    }
}
