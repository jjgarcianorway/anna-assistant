// v0.0.664: Settings Resolution (Phase 240)
// Resolve settings values with reference following and computation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ResolutionStrategy {
    /// Direct value lookup
    #[default]
    Direct,
    /// Follow references
    Reference,
    /// Compute from dependencies
    Computed,
    /// Use cached value
    Cached,
    /// Use default value
    Default,
}

impl std::fmt::Display for ResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Reference => write!(f, "reference"),
            Self::Computed => write!(f, "computed"),
            Self::Cached => write!(f, "cached"),
            Self::Default => write!(f, "default"),
        }
    }
}

/// Resolution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResolutionStatus {
    /// Resolved successfully
    #[default]
    Resolved,
    /// Pending resolution
    Pending,
    /// Failed to resolve
    Failed,
    /// Circular reference detected
    Circular,
    /// Not found
    NotFound,
}

impl std::fmt::Display for ResolutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolved => write!(f, "resolved"),
            Self::Pending => write!(f, "pending"),
            Self::Failed => write!(f, "failed"),
            Self::Circular => write!(f, "circular"),
            Self::NotFound => write!(f, "not_found"),
        }
    }
}

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

/// Resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    /// Key resolved
    pub key: String,
    /// Resolved value
    pub value: Option<String>,
    /// Status
    pub status: ResolutionStatus,
    /// Strategy used
    pub strategy: ResolutionStrategy,
    /// Depth of resolution
    pub depth: usize,
    /// Error message
    pub error: Option<String>,
}

impl ResolutionResult {
    /// Create success result
    pub fn success(key: impl Into<String>, value: impl Into<String>, strategy: ResolutionStrategy) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
            status: ResolutionStatus::Resolved,
            strategy,
            depth: 0,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(key: impl Into<String>, status: ResolutionStatus, error: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
            status,
            strategy: ResolutionStrategy::Direct,
            depth: 0,
            error: Some(error.into()),
        }
    }

    /// With depth
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Is resolved
    pub fn is_resolved(&self) -> bool {
        self.status == ResolutionStatus::Resolved
    }

    /// Is failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, ResolutionStatus::Failed | ResolutionStatus::Circular | ResolutionStatus::NotFound)
    }
}

/// Resolution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRequest {
    /// Key to resolve
    pub key: String,
    /// Strategy to use
    pub strategy: Option<ResolutionStrategy>,
    /// Default value
    pub default_value: Option<String>,
}

impl ResolutionRequest {
    /// Create new request
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            strategy: None,
            default_value: None,
        }
    }

    /// With strategy
    pub fn with_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// With default
    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

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
        self.resolve_with_depth(key, 0, &mut std::collections::HashSet::new())
    }

    /// Resolve with depth tracking
    fn resolve_with_depth(
        &mut self,
        key: &str,
        depth: usize,
        visited: &mut std::collections::HashSet<String>,
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

/// Settings resolver registry
#[derive(Debug, Clone, Default)]
pub struct SettingsResolverRegistry {
    /// Resolvers by ID
    resolvers: HashMap<String, SettingsResolver>,
}

impl SettingsResolverRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register resolver
    pub fn register(&mut self, id: impl Into<String>, resolver: SettingsResolver) {
        self.resolvers.insert(id.into(), resolver);
    }

    /// Unregister resolver
    pub fn unregister(&mut self, id: &str) -> bool {
        self.resolvers.remove(id).is_some()
    }

    /// Get resolver
    pub fn get(&self, id: &str) -> Option<&SettingsResolver> {
        self.resolvers.get(id)
    }

    /// Get resolver mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsResolver> {
        self.resolvers.get_mut(id)
    }

    /// Resolver count
    pub fn count(&self) -> usize {
        self.resolvers.len()
    }
}

/// Format resolver registry
pub fn format_resolver_registry(registry: &SettingsResolverRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Resolver Registry:\n");
    output.push_str(&format!("  Resolvers: {}\n", registry.count()));
    output
}

/// Check if query is about resolver
pub fn is_resolver_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("resolve") || lower.contains("settings resolver") || lower.contains("resolution")
}

/// Fun fact about resolver
pub fn resolver_fun_fact() -> &'static str {
    "Anna's settings resolver follows references to find the final values!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_display() {
        assert_eq!(format!("{}", ResolutionStrategy::Direct), "direct");
        assert_eq!(format!("{}", ResolutionStrategy::Reference), "reference");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ResolutionStatus::Resolved), "resolved");
        assert_eq!(format!("{}", ResolutionStatus::Circular), "circular");
    }

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

    #[test]
    fn test_result_success() {
        let r = ResolutionResult::success("key", "value", ResolutionStrategy::Direct);
        assert!(r.is_resolved());
        assert!(!r.is_failed());
    }

    #[test]
    fn test_result_failure() {
        let r = ResolutionResult::failure("key", ResolutionStatus::NotFound, "not found");
        assert!(!r.is_resolved());
        assert!(r.is_failed());
    }

    #[test]
    fn test_request_new() {
        let r = ResolutionRequest::new("key");
        assert_eq!(r.key, "key");
    }

    #[test]
    fn test_request_with_default() {
        let r = ResolutionRequest::new("key").with_default("default");
        assert_eq!(r.default_value, Some("default".to_string()));
    }

    #[test]
    fn test_stats_record() {
        let mut s = ResolverStats::default();
        let r = ResolutionResult::success("k", "v", ResolutionStrategy::Direct);
        s.record(&r);
        assert_eq!(s.total_resolutions, 1);
        assert_eq!(s.successful, 1);
    }

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

    #[test]
    fn test_registry_new() {
        let r = SettingsResolverRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsResolverRegistry::new();
        r.register("r1", SettingsResolver::new(ResolverConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_resolver_query() {
        assert!(is_resolver_query("resolve settings"));
        assert!(!is_resolver_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = resolver_fun_fact();
        assert!(fact.contains("resolve"));
    }
}
