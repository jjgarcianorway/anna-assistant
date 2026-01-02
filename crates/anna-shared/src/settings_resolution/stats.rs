// v0.0.664: Settings Resolution Stats
// Statistics tracking for resolver

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::result::ResolutionResult;

/// Resolver stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolverStats {
    /// Total resolutions
    pub total_resolutions: usize,
    /// Successful resolutions
    pub successful: usize,
    /// Failed resolutions
    pub failed: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// By strategy
    pub by_strategy: HashMap<String, usize>,
}

impl ResolverStats {
    /// Record resolution
    pub fn record(&mut self, result: &ResolutionResult) {
        self.total_resolutions += 1;
        if result.is_resolved() {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        *self.by_strategy.entry(result.strategy.to_string()).or_insert(0) += 1;
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_resolutions == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_resolutions as f64
        }
    }

    /// Cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_resolutions == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_resolutions as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_resolution::types::ResolutionStrategy;

    #[test]
    fn test_stats_record() {
        let mut s = ResolverStats::default();
        let r = ResolutionResult::success("k", "v", ResolutionStrategy::Direct);
        s.record(&r);
        assert_eq!(s.total_resolutions, 1);
        assert_eq!(s.successful, 1);
    }
}
