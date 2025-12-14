// v0.0.626: Settings Proxy (Phase 202)
// Proxy layer for settings access with caching and transformation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Proxy behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProxyBehavior {
    /// Passthrough - no caching
    #[default]
    Passthrough,
    /// Cache - cache responses
    Cache,
    /// Transform - apply transformations
    Transform,
    /// Intercept - intercept and modify
    Intercept,
}

impl std::fmt::Display for ProxyBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passthrough => write!(f, "passthrough"),
            Self::Cache => write!(f, "cache"),
            Self::Transform => write!(f, "transform"),
            Self::Intercept => write!(f, "intercept"),
        }
    }
}

/// Cache status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CacheStatus {
    /// Hit
    Hit,
    /// Miss
    #[default]
    Miss,
    /// Stale
    Stale,
    /// Bypass
    Bypass,
}

impl std::fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hit => write!(f, "hit"),
            Self::Miss => write!(f, "miss"),
            Self::Stale => write!(f, "stale"),
            Self::Bypass => write!(f, "bypass"),
        }
    }
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Category
    pub category: SettingsCategory,
    /// Created timestamp
    pub created_at: u64,
    /// TTL seconds
    pub ttl_seconds: u64,
}

impl CacheEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, category: SettingsCategory) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            category,
            created_at: 0,
            ttl_seconds: 300, // 5 minutes default
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Set TTL
    pub fn ttl(mut self, seconds: u64) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    /// Is expired
    pub fn is_expired(&self, now: u64) -> bool {
        now > self.created_at + self.ttl_seconds
    }
}

/// Proxy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyOperation {
    /// Operation ID
    pub id: String,
    /// Key
    pub key: String,
    /// Cache status
    pub cache_status: CacheStatus,
    /// Transformed
    pub transformed: bool,
    /// Duration ms
    pub duration_ms: u64,
}

impl ProxyOperation {
    /// Create new operation
    pub fn new(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            cache_status: CacheStatus::Miss,
            transformed: false,
            duration_ms: 0,
        }
    }

    /// Set cache status
    pub fn cache_status(mut self, status: CacheStatus) -> Self {
        self.cache_status = status;
        self
    }

    /// Mark transformed
    pub fn mark_transformed(&mut self) {
        self.transformed = true;
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Proxy statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyStats {
    /// Total operations
    pub total_operations: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Transformations
    pub transformations: usize,
}

impl ProxyStats {
    /// Record operation
    pub fn record(&mut self, cache_hit: bool, transformed: bool) {
        self.total_operations += 1;
        if cache_hit {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
        if transformed {
            self.transformations += 1;
        }
    }

    /// Cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Settings proxy
#[derive(Debug, Clone, Default)]
pub struct SettingsProxy {
    /// Behavior
    behavior: ProxyBehavior,
    /// Cache
    cache: HashMap<String, CacheEntry>,
    /// Statistics
    stats: ProxyStats,
    /// Max cache size
    max_cache_size: usize,
}

impl SettingsProxy {
    /// Create new proxy
    pub fn new() -> Self {
        Self {
            max_cache_size: 1000,
            ..Default::default()
        }
    }

    /// Get behavior
    pub fn behavior(&self) -> ProxyBehavior {
        self.behavior
    }

    /// Set behavior
    pub fn set_behavior(&mut self, behavior: ProxyBehavior) {
        self.behavior = behavior;
    }

    /// Get from cache
    pub fn get_cached(&mut self, key: &str, now: u64) -> Option<&CacheEntry> {
        if let Some(entry) = self.cache.get(key) {
            if !entry.is_expired(now) {
                self.stats.record(true, false);
                return Some(entry);
            }
        }
        self.stats.record(false, false);
        None
    }

    /// Put in cache
    pub fn put_cached(&mut self, entry: CacheEntry) {
        if self.cache.len() >= self.max_cache_size {
            // Remove oldest entry (simple eviction)
            if let Some(oldest_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(entry.key.clone(), entry);
    }

    /// Invalidate cache entry
    pub fn invalidate(&mut self, key: &str) -> bool {
        self.cache.remove(key).is_some()
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get stats
    pub fn stats(&self) -> &ProxyStats {
        &self.stats
    }
}

/// Format proxy
pub fn format_proxy(proxy: &SettingsProxy) -> String {
    let mut output = String::new();
    output.push_str("Settings Proxy:\n");
    output.push_str(&format!("  Behavior: {}\n", proxy.behavior()));
    output.push_str(&format!("  Cache Size: {}\n", proxy.cache_size()));
    output.push_str(&format!("  Hit Rate: {:.1}%\n", proxy.stats().cache_hit_rate() * 100.0));
    output
}

/// Check if query is about proxy
pub fn is_proxy_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("proxy")
        || lower.contains("settings proxy")
        || lower.contains("cache proxy")
}

/// Fun fact about proxy
pub fn proxy_fun_fact() -> &'static str {
    "Anna's settings proxy provides caching and transformation for faster access!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_display() {
        assert_eq!(format!("{}", ProxyBehavior::Passthrough), "passthrough");
        assert_eq!(format!("{}", ProxyBehavior::Cache), "cache");
    }

    #[test]
    fn test_cache_status_display() {
        assert_eq!(format!("{}", CacheStatus::Hit), "hit");
        assert_eq!(format!("{}", CacheStatus::Miss), "miss");
    }

    #[test]
    fn test_cache_entry_new() {
        let e = CacheEntry::new("key", "value", SettingsCategory::Privacy);
        assert_eq!(e.ttl_seconds, 300);
    }

    #[test]
    fn test_cache_entry_expired() {
        let e = CacheEntry::new("key", "value", SettingsCategory::Privacy)
            .created_at(100)
            .ttl(60);
        assert!(e.is_expired(200));
    }

    #[test]
    fn test_operation_new() {
        let o = ProxyOperation::new("op1", "key1");
        assert!(!o.transformed);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ProxyStats::default();
        s.record(true, false);
        assert_eq!(s.cache_hits, 1);
    }

    #[test]
    fn test_proxy_new() {
        let p = SettingsProxy::new();
        assert_eq!(p.cache_size(), 0);
    }

    #[test]
    fn test_proxy_put_get() {
        let mut p = SettingsProxy::new();
        p.put_cached(CacheEntry::new("k1", "v1", SettingsCategory::Privacy).created_at(100));
        assert!(p.get_cached("k1", 200).is_some());
    }

    #[test]
    fn test_proxy_invalidate() {
        let mut p = SettingsProxy::new();
        p.put_cached(CacheEntry::new("k1", "v1", SettingsCategory::Privacy));
        assert!(p.invalidate("k1"));
    }

    #[test]
    fn test_proxy_clear() {
        let mut p = SettingsProxy::new();
        p.put_cached(CacheEntry::new("k1", "v1", SettingsCategory::Privacy));
        p.clear_cache();
        assert_eq!(p.cache_size(), 0);
    }

    #[test]
    fn test_is_proxy_query() {
        assert!(is_proxy_query("settings proxy"));
        assert!(!is_proxy_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = proxy_fun_fact();
        assert!(fact.contains("proxy"));
    }
}
