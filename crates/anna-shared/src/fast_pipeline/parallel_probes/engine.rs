//! Parallel Probe Engine - v0.0.438.
//!
//! Execute probes in parallel with smart caching.

use super::cache::{CacheStats, ProbeCache};
use super::types::{
    PreparedBatch, ProbeResult, MAX_CONCURRENT_PROBES, PROBE_TIMEOUT_MS,
};
use std::time::Duration;

/// Parallel probe execution engine.
#[derive(Debug)]
pub struct ParallelProbeEngine {
    /// Cache for probe results.
    pub cache: ProbeCache,
    /// Max concurrent probes.
    pub max_concurrent: usize,
    /// Default timeout per probe.
    pub default_timeout: Duration,
}

impl ParallelProbeEngine {
    /// Create new engine.
    pub fn new() -> Self {
        Self {
            cache: ProbeCache::new(),
            max_concurrent: MAX_CONCURRENT_PROBES,
            default_timeout: Duration::from_millis(PROBE_TIMEOUT_MS),
        }
    }

    /// Create with custom settings.
    pub fn with_config(max_concurrent: usize, timeout_ms: u64, cache_ttl_s: u64) -> Self {
        Self {
            cache: ProbeCache::with_ttl(cache_ttl_s),
            max_concurrent,
            default_timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Check cache for probe.
    pub fn check_cache(&mut self, probe_id: &str) -> Option<ProbeResult> {
        self.cache
            .get(probe_id)
            .map(|v| ProbeResult::cached(probe_id, &v))
    }

    /// Cache a result.
    pub fn cache_result(&mut self, probe_id: &str, value: &str) {
        self.cache.set(probe_id, value);
    }

    /// Prepare batch execution - returns which need execution vs cached.
    pub fn prepare_batch(&mut self, probe_ids: &[String]) -> PreparedBatch {
        let mut cached = Vec::new();
        let mut to_execute = Vec::new();

        for id in probe_ids {
            if let Some(result) = self.check_cache(id) {
                cached.push(result);
            } else {
                to_execute.push(id.clone());
            }
        }

        PreparedBatch {
            cached,
            to_execute,
            concurrency: self.max_concurrent,
            timeout: self.default_timeout,
        }
    }

    /// Get cache stats.
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Cleanup expired cache.
    pub fn cleanup_cache(&mut self) {
        self.cache.cleanup();
    }
}

impl Default for ParallelProbeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::ProbeStatus;
    use super::*;

    #[test]
    fn test_parallel_probe_engine() {
        let mut engine = ParallelProbeEngine::new();

        // No cache initially
        assert!(engine.check_cache("sys.mem").is_none());

        // Cache a result
        engine.cache_result("sys.mem", "4 GB");
        let cached = engine.check_cache("sys.mem");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().status, ProbeStatus::Cached);
    }

    #[test]
    fn test_prepare_batch() {
        let mut engine = ParallelProbeEngine::new();
        engine.cache_result("a", "1");

        let prepared = engine.prepare_batch(&["a".into(), "b".into(), "c".into()]);
        assert_eq!(prepared.cached.len(), 1);
        assert_eq!(prepared.to_execute.len(), 2);
        assert!(prepared.needs_execution());
    }
}
