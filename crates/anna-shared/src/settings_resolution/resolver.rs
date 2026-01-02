// v0.0.664: Settings Resolver
// Core resolver implementation

use std::collections::{HashMap, HashSet};
use super::config::ResolverConfig;
use super::result::{ResolutionResult, ResolutionRequest};
use super::stats::ResolverStats;
use super::types::{ResolutionStatus, ResolutionStrategy};

/// Settings resolver
#[derive(Debug, Clone, Default)]
pub struct SettingsResolver {
    /// Config
    config: ResolverConfig,
    /// Settings storage
    settings: HashMap<String, String>,
    /// References map
    references: HashMap<String, String>,
    /// Cache
    cache: HashMap<String, String>,
    /// Stats
    stats: ResolverStats,
}

impl SettingsResolver {
    /// Create new resolver
    pub fn new(config: ResolverConfig) -> Self {
        Self {
            config,
            settings: HashMap::new(),
            references: HashMap::new(),
            cache: HashMap::new(),
            stats: ResolverStats::default(),
        }
    }

    /// Set value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Set reference
    pub fn set_reference(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.references.insert(from.into(), to.into());
    }

    /// Resolve key
    pub fn resolve(&mut self, key: &str) -> ResolutionResult {
        self.resolve_with_depth(key, 0, &mut HashSet::new())
    }

    /// Resolve with depth tracking
    fn resolve_with_depth(
        &mut self,
        key: &str,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> ResolutionResult {
        // Check max depth
        if depth > self.config.max_depth {
            let result = ResolutionResult::failure(key, ResolutionStatus::Failed, "Max depth exceeded");
            self.stats.record(&result);
            return result;
        }

        // Check circular
        if visited.contains(key) {
            let result = ResolutionResult::failure(key, ResolutionStatus::Circular, "Circular reference detected");
            self.stats.record(&result);
            return result;
        }

        // Check cache
        if self.config.enable_cache {
            if let Some(cached) = self.cache.get(key) {
                self.stats.record_cache_hit();
                return ResolutionResult::success(key, cached.clone(), ResolutionStrategy::Cached)
                    .with_depth(depth);
            }
        }

        visited.insert(key.to_string());

        // Try direct lookup
        if let Some(value) = self.settings.get(key) {
            let result = ResolutionResult::success(key, value.clone(), ResolutionStrategy::Direct)
                .with_depth(depth);
            if self.config.enable_cache {
                self.cache.insert(key.to_string(), value.clone());
            }
            self.stats.record(&result);
            return result;
        }

        // Try reference
        if let Some(ref_key) = self.references.get(key).cloned() {
            let ref_result = self.resolve_with_depth(&ref_key, depth + 1, visited);
            if ref_result.is_resolved() {
                let value = ref_result.value.unwrap();
                let result = ResolutionResult::success(key, value.clone(), ResolutionStrategy::Reference)
                    .with_depth(depth);
                if self.config.enable_cache {
                    self.cache.insert(key.to_string(), value);
                }
                self.stats.record(&result);
                return result;
            }
            // Propagate circular error
            if ref_result.status == ResolutionStatus::Circular {
                return ref_result;
            }
        }

        // Not found
        let result = ResolutionResult::failure(key, ResolutionStatus::NotFound, "Key not found");
        self.stats.record(&result);
        result
    }

    /// Resolve request
    pub fn resolve_request(&mut self, request: &ResolutionRequest) -> ResolutionResult {
        let result = self.resolve(&request.key);
        if result.is_failed() && request.default_value.is_some() {
            return ResolutionResult::success(
                &request.key,
                request.default_value.clone().unwrap(),
                ResolutionStrategy::Default,
            );
        }
        result
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get stats
    pub fn stats(&self) -> &ResolverStats {
        &self.stats
    }

    /// Settings count
    pub fn settings_count(&self) -> usize {
        self.settings.len()
    }

    /// References count
    pub fn references_count(&self) -> usize {
        self.references.len()
    }

    /// Cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_new() {
        let r = SettingsResolver::new(ResolverConfig::default());
        assert_eq!(r.settings_count(), 0);
    }

    #[test]
    fn test_resolver_set_resolve() {
        let mut r = SettingsResolver::new(ResolverConfig::default());
        r.set("key", "value");
        let result = r.resolve("key");
        assert!(result.is_resolved());
        assert_eq!(result.value, Some("value".to_string()));
    }

    #[test]
    fn test_resolver_reference() {
        let mut r = SettingsResolver::new(ResolverConfig::default());
        r.set("actual", "value");
        r.set_reference("alias", "actual");
        let result = r.resolve("alias");
        assert!(result.is_resolved());
        assert_eq!(result.value, Some("value".to_string()));
    }

    #[test]
    fn test_resolver_circular_detection() {
        let mut r = SettingsResolver::new(ResolverConfig::default());
        r.set_reference("a", "b");
        r.set_reference("b", "c");
        r.set_reference("c", "a"); // Creates cycle: a -> b -> c -> a
        let result = r.resolve("a");
        assert_eq!(result.status, ResolutionStatus::Circular);
    }

    #[test]
    fn test_resolver_request_default() {
        let mut r = SettingsResolver::new(ResolverConfig::default());
        let req = ResolutionRequest::new("missing").with_default("default_val");
        let result = r.resolve_request(&req);
        assert_eq!(result.value, Some("default_val".to_string()));
    }
}
