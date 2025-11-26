//! TTL-based cache for state and probe results

use anna_common::{CachePolicy, ProbeResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

/// Cached entry with expiration
struct CacheEntry {
    value: serde_json::Value,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn new(value: serde_json::Value, ttl: Option<Duration>) -> Self {
        Self {
            value,
            expires_at: ttl.map(|d| Instant::now() + d),
        }
    }

    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Instant::now() > exp)
    }
}

/// State manager with TTL-based caching
pub struct StateManager {
    state: HashMap<String, CacheEntry>,
    probe_cache: HashMap<String, (ProbeResult, Option<Instant>)>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            probe_cache: HashMap::new(),
        }
    }

    /// Get a value from state
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.state.get(key).and_then(|entry| {
            if entry.is_expired() {
                debug!("  Cache expired: {}", key);
                None
            } else {
                Some(entry.value.clone())
            }
        })
    }

    /// Set a value in state
    pub fn set(&mut self, key: &str, value: serde_json::Value, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.map(Duration::from_secs);
        self.state
            .insert(key.to_string(), CacheEntry::new(value, ttl));
    }

    /// Invalidate a specific key
    pub fn invalidate(&mut self, key: &str) {
        self.state.remove(key);
        self.probe_cache.remove(key);
    }

    /// Invalidate keys matching a pattern (simple prefix match)
    pub fn invalidate_pattern(&mut self, pattern: &str) {
        self.state.retain(|k, _| !k.starts_with(pattern));
        self.probe_cache.retain(|k, _| !k.starts_with(pattern));
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.state.clear();
        self.probe_cache.clear();
    }

    /// Get cached probe result
    pub fn get_probe_cache(&self, probe_id: &str) -> Option<ProbeResult> {
        self.probe_cache.get(probe_id).and_then(|(result, exp)| {
            if exp.map_or(false, |e| Instant::now() > e) {
                debug!("  Probe cache expired: {}", probe_id);
                None
            } else {
                let mut cached_result = result.clone();
                cached_result.cached = true;
                Some(cached_result)
            }
        })
    }

    /// Set probe result in cache
    pub fn set_probe_cache(&mut self, probe_id: &str, result: &ProbeResult, policy: CachePolicy) {
        let ttl_secs = policy.ttl_seconds();
        let expires_at = if ttl_secs == 0 {
            None // Static = never expires
        } else {
            Some(Instant::now() + Duration::from_secs(ttl_secs))
        };
        self.probe_cache
            .insert(probe_id.to_string(), (result.clone(), expires_at));
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut mgr = StateManager::new();
        mgr.set("key1", serde_json::json!("value1"), None);
        assert_eq!(mgr.get("key1"), Some(serde_json::json!("value1")));
    }

    #[test]
    fn test_invalidate() {
        let mut mgr = StateManager::new();
        mgr.set("key1", serde_json::json!("value1"), None);
        mgr.invalidate("key1");
        assert_eq!(mgr.get("key1"), None);
    }

    #[test]
    fn test_invalidate_pattern() {
        let mut mgr = StateManager::new();
        mgr.set("cpu.info", serde_json::json!("cpu"), None);
        mgr.set("cpu.load", serde_json::json!("load"), None);
        mgr.set("mem.info", serde_json::json!("mem"), None);
        mgr.invalidate_pattern("cpu.");
        assert_eq!(mgr.get("cpu.info"), None);
        assert_eq!(mgr.get("cpu.load"), None);
        assert_eq!(mgr.get("mem.info"), Some(serde_json::json!("mem")));
    }
}
