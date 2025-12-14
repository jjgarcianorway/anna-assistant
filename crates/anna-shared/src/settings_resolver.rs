// v0.0.599: Settings Resolver (Phase 175)
// Resolution logic for settings conflicts and dependencies

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::unified_settings::SettingsCategory;

/// Conflict type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Value mismatch
    ValueMismatch,
    /// Type mismatch
    TypeMismatch,
    /// Dependency missing
    DependencyMissing,
    /// Circular dependency
    CircularDep,
    /// Mutual exclusion
    MutualExclusion,
    /// Version conflict
    VersionConflict,
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueMismatch => write!(f, "value_mismatch"),
            Self::TypeMismatch => write!(f, "type_mismatch"),
            Self::DependencyMissing => write!(f, "dependency_missing"),
            Self::CircularDep => write!(f, "circular_dependency"),
            Self::MutualExclusion => write!(f, "mutual_exclusion"),
            Self::VersionConflict => write!(f, "version_conflict"),
        }
    }
}

/// Resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Use first value
    First,
    /// Use last value
    Last,
    /// Use highest priority
    Priority,
    /// Merge values
    Merge,
    /// Fail on conflict
    Fail,
    /// Ask user
    Ask,
}

impl std::fmt::Display for ResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::Last => write!(f, "last"),
            Self::Priority => write!(f, "priority"),
            Self::Merge => write!(f, "merge"),
            Self::Fail => write!(f, "fail"),
            Self::Ask => write!(f, "ask"),
        }
    }
}

/// Conflict description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Source A
    pub source_a: String,
    /// Source B
    pub source_b: String,
    /// Affected key
    pub key: String,
    /// Category
    pub category: SettingsCategory,
    /// Description
    pub description: String,
}

impl Conflict {
    /// Create new conflict
    pub fn new(
        conflict_type: ConflictType,
        source_a: impl Into<String>,
        source_b: impl Into<String>,
        key: impl Into<String>,
        category: SettingsCategory,
    ) -> Self {
        Self {
            conflict_type,
            source_a: source_a.into(),
            source_b: source_b.into(),
            key: key.into(),
            category,
            description: String::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    /// Conflict
    pub conflict: Conflict,
    /// Strategy used
    pub strategy: ResolutionStrategy,
    /// Resolved value
    pub resolved_value: Option<String>,
    /// Success
    pub success: bool,
    /// Notes
    pub notes: String,
}

impl Resolution {
    /// Create successful resolution
    pub fn success(conflict: Conflict, strategy: ResolutionStrategy, value: impl Into<String>) -> Self {
        Self {
            conflict,
            strategy,
            resolved_value: Some(value.into()),
            success: true,
            notes: String::new(),
        }
    }

    /// Create failed resolution
    pub fn failure(conflict: Conflict, notes: impl Into<String>) -> Self {
        Self {
            conflict,
            strategy: ResolutionStrategy::Fail,
            resolved_value: None,
            success: false,
            notes: notes.into(),
        }
    }
}

/// Dependency entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Source key
    pub source: String,
    /// Depends on
    pub depends_on: String,
    /// Required
    pub required: bool,
    /// Category
    pub category: SettingsCategory,
}

impl Dependency {
    /// Create new dependency
    pub fn new(
        source: impl Into<String>,
        depends_on: impl Into<String>,
        category: SettingsCategory,
    ) -> Self {
        Self {
            source: source.into(),
            depends_on: depends_on.into(),
            required: true,
            category,
        }
    }

    /// Set optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Resolver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverConfig {
    /// Default strategy
    pub default_strategy: ResolutionStrategy,
    /// Per-category strategies
    pub category_strategies: HashMap<SettingsCategory, ResolutionStrategy>,
    /// Auto-resolve
    pub auto_resolve: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            default_strategy: ResolutionStrategy::Last,
            category_strategies: HashMap::new(),
            auto_resolve: true,
        }
    }
}

impl ResolverConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default strategy
    pub fn default_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Set category strategy
    pub fn category_strategy(mut self, category: SettingsCategory, strategy: ResolutionStrategy) -> Self {
        self.category_strategies.insert(category, strategy);
        self
    }

    /// Get strategy for category
    pub fn strategy_for(&self, category: SettingsCategory) -> ResolutionStrategy {
        self.category_strategies
            .get(&category)
            .copied()
            .unwrap_or(self.default_strategy)
    }
}

/// Settings resolver
#[derive(Debug, Clone, Default)]
pub struct SettingsResolver {
    /// Configuration
    config: ResolverConfig,
    /// Dependencies
    dependencies: Vec<Dependency>,
    /// Conflict history
    conflicts: Vec<Conflict>,
    /// Resolution history
    resolutions: Vec<Resolution>,
}

impl SettingsResolver {
    /// Create new resolver
    pub fn new() -> Self {
        Self::default()
    }

    /// With config
    pub fn with_config(config: ResolverConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Add dependency
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    /// Remove dependency
    pub fn remove_dependency(&mut self, source: &str, depends_on: &str) -> bool {
        if let Some(pos) = self
            .dependencies
            .iter()
            .position(|d| d.source == source && d.depends_on == depends_on)
        {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get dependencies for key
    pub fn dependencies_for(&self, source: &str) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| d.source == source).collect()
    }

    /// Get dependents of key
    pub fn dependents_of(&self, key: &str) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| d.depends_on == key).collect()
    }

    /// Check for circular dependencies
    pub fn has_circular(&self, start: &str) -> bool {
        let mut visited = HashSet::new();
        self.check_circular_internal(start, &mut visited)
    }

    fn check_circular_internal(&self, current: &str, visited: &mut HashSet<String>) -> bool {
        if visited.contains(current) {
            return true;
        }
        visited.insert(current.to_string());

        for dep in self.dependencies_for(current) {
            if self.check_circular_internal(&dep.depends_on, visited) {
                return true;
            }
        }
        visited.remove(current);
        false
    }

    /// Record conflict
    pub fn record_conflict(&mut self, conflict: Conflict) {
        self.conflicts.push(conflict);
    }

    /// Record resolution
    pub fn record_resolution(&mut self, resolution: Resolution) {
        self.resolutions.push(resolution);
    }

    /// Get conflicts
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Get resolutions
    pub fn resolutions(&self) -> &[Resolution] {
        &self.resolutions
    }

    /// Conflict count
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Resolution count
    pub fn resolution_count(&self) -> usize {
        self.resolutions.len()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.conflicts.clear();
        self.resolutions.clear();
    }
}

/// Format resolver
pub fn format_resolver(resolver: &SettingsResolver) -> String {
    let mut output = String::new();
    output.push_str("Settings Resolver:\n");
    output.push_str(&format!("  Dependencies: {}\n", resolver.dependencies.len()));
    output.push_str(&format!("  Conflicts: {}\n", resolver.conflict_count()));
    output.push_str(&format!("  Resolutions: {}\n", resolver.resolution_count()));
    output
}

/// Check if query is about resolver
pub fn is_resolver_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("resolve")
        || lower.contains("conflict")
        || lower.contains("dependency")
}

/// Fun fact about resolver
pub fn resolver_fun_fact() -> &'static str {
    "Anna automatically resolves settings conflicts based on configurable strategies!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_type_display() {
        assert_eq!(format!("{}", ConflictType::ValueMismatch), "value_mismatch");
        assert_eq!(format!("{}", ConflictType::CircularDep), "circular_dependency");
    }

    #[test]
    fn test_resolution_strategy_display() {
        assert_eq!(format!("{}", ResolutionStrategy::First), "first");
        assert_eq!(format!("{}", ResolutionStrategy::Merge), "merge");
    }

    #[test]
    fn test_conflict_new() {
        let c = Conflict::new(
            ConflictType::ValueMismatch,
            "a", "b", "key",
            SettingsCategory::Personality,
        );
        assert_eq!(c.key, "key");
    }

    #[test]
    fn test_resolution_success() {
        let c = Conflict::new(
            ConflictType::ValueMismatch,
            "a", "b", "k",
            SettingsCategory::Privacy,
        );
        let r = Resolution::success(c, ResolutionStrategy::Last, "value");
        assert!(r.success);
    }

    #[test]
    fn test_dependency_new() {
        let d = Dependency::new("a", "b", SettingsCategory::Risk);
        assert!(d.required);
    }

    #[test]
    fn test_dependency_optional() {
        let d = Dependency::new("a", "b", SettingsCategory::Risk).optional();
        assert!(!d.required);
    }

    #[test]
    fn test_config_default() {
        let c = ResolverConfig::new();
        assert_eq!(c.default_strategy, ResolutionStrategy::Last);
    }

    #[test]
    fn test_config_category_strategy() {
        let c = ResolverConfig::new()
            .category_strategy(SettingsCategory::Personality, ResolutionStrategy::First);
        assert_eq!(c.strategy_for(SettingsCategory::Personality), ResolutionStrategy::First);
    }

    #[test]
    fn test_resolver_new() {
        let r = SettingsResolver::new();
        assert_eq!(r.conflict_count(), 0);
    }

    #[test]
    fn test_resolver_dependencies() {
        let mut r = SettingsResolver::new();
        r.add_dependency(Dependency::new("a", "b", SettingsCategory::Privacy));
        assert_eq!(r.dependencies_for("a").len(), 1);
    }

    #[test]
    fn test_resolver_circular() {
        let mut r = SettingsResolver::new();
        r.add_dependency(Dependency::new("a", "b", SettingsCategory::Privacy));
        r.add_dependency(Dependency::new("b", "a", SettingsCategory::Privacy));
        assert!(r.has_circular("a"));
    }

    #[test]
    fn test_is_resolver_query() {
        assert!(is_resolver_query("resolve conflict"));
        assert!(!is_resolver_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = resolver_fun_fact();
        assert!(fact.contains("resolve"));
    }
}
