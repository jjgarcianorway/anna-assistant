// v0.0.670: Settings Query Engine Config (Phase 246)
// Configuration for query engine

use serde::{Deserialize, Serialize};

/// Query engine config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEngineConfig {
    /// Max results
    pub max_results: usize,
    /// Enable caching
    pub enable_cache: bool,
    /// Timeout (ms)
    pub timeout_ms: u64,
    /// Case insensitive
    pub case_insensitive: bool,
}

impl QueryEngineConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            max_results: 1000,
            enable_cache: true,
            timeout_ms: 5000,
            case_insensitive: true,
        }
    }

    /// Set max results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Set enable cache
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }
}

impl Default for QueryEngineConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = QueryEngineConfig::new();
        assert!(c.enable_cache);
    }

    #[test]
    fn test_config_builder() {
        let c = QueryEngineConfig::new()
            .max_results(100)
            .case_insensitive(false);
        assert_eq!(c.max_results, 100);
        assert!(!c.case_insensitive);
    }
}
