// v0.0.664: Settings Resolution Config
// Configuration for the resolver

use serde::{Deserialize, Serialize};
use super::types::ResolutionStrategy;

/// Resolver config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverConfig {
    /// Default strategy
    pub default_strategy: ResolutionStrategy,
    /// Max reference depth
    pub max_depth: usize,
    /// Enable caching
    pub enable_cache: bool,
    /// Fail on circular
    pub fail_on_circular: bool,
    /// Use defaults on failure
    pub use_defaults: bool,
}

impl ResolverConfig {
    /// Create new config
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            default_strategy: strategy,
            max_depth: 10,
            enable_cache: true,
            fail_on_circular: true,
            use_defaults: true,
        }
    }

    /// Set max depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set enable cache
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Set fail on circular
    pub fn fail_on_circular(mut self, fail: bool) -> Self {
        self.fail_on_circular = fail;
        self
    }

    /// Set use defaults
    pub fn use_defaults(mut self, use_def: bool) -> Self {
        self.use_defaults = use_def;
        self
    }
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self::new(ResolutionStrategy::Direct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ResolverConfig::new(ResolutionStrategy::Direct);
        assert_eq!(c.max_depth, 10);
        assert!(c.enable_cache);
    }

    #[test]
    fn test_config_builder() {
        let c = ResolverConfig::new(ResolutionStrategy::Reference)
            .max_depth(5)
            .enable_cache(false);
        assert_eq!(c.max_depth, 5);
        assert!(!c.enable_cache);
    }
}
