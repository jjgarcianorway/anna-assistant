// v0.0.654: Settings Injector (Phase 230)
// Injector for inserting settings into configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Injection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum InjectionType {
    /// Insert new value
    #[default]
    Insert,
    /// Update existing value
    Update,
    /// Upsert (insert or update)
    Upsert,
    /// Replace all
    Replace,
    /// Append to existing
    Append,
}

impl std::fmt::Display for InjectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "insert"),
            Self::Update => write!(f, "update"),
            Self::Upsert => write!(f, "upsert"),
            Self::Replace => write!(f, "replace"),
            Self::Append => write!(f, "append"),
        }
    }
}

/// Injection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InjectionStrategy {
    /// Fail on conflict
    #[default]
    FailOnConflict,
    /// Skip on conflict
    SkipOnConflict,
    /// Overwrite on conflict
    OverwriteOnConflict,
    /// Merge on conflict
    MergeOnConflict,
}

impl std::fmt::Display for InjectionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailOnConflict => write!(f, "fail_on_conflict"),
            Self::SkipOnConflict => write!(f, "skip_on_conflict"),
            Self::OverwriteOnConflict => write!(f, "overwrite_on_conflict"),
            Self::MergeOnConflict => write!(f, "merge_on_conflict"),
        }
    }
}

/// Injector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectorConfig {
    /// Injection type
    pub injection_type: InjectionType,
    /// Injection strategy
    pub strategy: InjectionStrategy,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before inject
    pub validate_before: bool,
    /// Dry run mode
    pub dry_run: bool,
}

impl InjectorConfig {
    /// Create new config
    pub fn new(injection_type: InjectionType) -> Self {
        Self {
            injection_type,
            strategy: InjectionStrategy::FailOnConflict,
            category: None,
            validate_before: true,
            dry_run: false,
        }
    }

    /// Set strategy
    pub fn strategy(mut self, strategy: InjectionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set validate before
    pub fn validate_before(mut self, validate: bool) -> Self {
        self.validate_before = validate;
        self
    }

    /// Set dry run
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self::new(InjectionType::Upsert)
    }
}

/// Injection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionResult {
    /// Keys inserted
    pub inserted: Vec<String>,
    /// Keys updated
    pub updated: Vec<String>,
    /// Keys skipped
    pub skipped: Vec<String>,
    /// Keys failed
    pub failed: Vec<String>,
    /// Injection type used
    pub injection_type: InjectionType,
}

impl InjectionResult {
    /// Create new result
    pub fn new(injection_type: InjectionType) -> Self {
        Self {
            inserted: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            injection_type,
        }
    }

    /// Add inserted
    pub fn add_inserted(&mut self, key: String) {
        self.inserted.push(key);
    }

    /// Add updated
    pub fn add_updated(&mut self, key: String) {
        self.updated.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.failed.push(key);
    }

    /// Total affected
    pub fn total_affected(&self) -> usize {
        self.inserted.len() + self.updated.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
}

/// Injector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InjectorStats {
    /// Total injections
    pub total_injections: usize,
    /// Total inserted
    pub total_inserted: usize,
    /// Total updated
    pub total_updated: usize,
    /// Total skipped
    pub total_skipped: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl InjectorStats {
    /// Record injection
    pub fn record(&mut self, injection_type: InjectionType, inserted: usize, updated: usize, skipped: usize) {
        self.total_injections += 1;
        self.total_inserted += inserted;
        self.total_updated += updated;
        self.total_skipped += skipped;
        *self.by_type.entry(injection_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_inserted + self.total_updated + self.total_skipped;
        if total == 0 {
            0.0
        } else {
            (self.total_inserted + self.total_updated) as f64 / total as f64
        }
    }
}

/// Settings injector
#[derive(Debug, Clone, Default)]
pub struct SettingsInjector {
    /// Config
    config: InjectorConfig,
    /// Results
    results: Vec<InjectionResult>,
    /// Stats
    stats: InjectorStats,
}

impl SettingsInjector {
    /// Create new injector
    pub fn new(config: InjectorConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: InjectorStats::default(),
        }
    }

    /// Inject settings
    pub fn inject(&mut self, target: &mut HashMap<String, String>, source: &HashMap<String, String>) -> InjectionResult {
        let mut result = InjectionResult::new(self.config.injection_type);

        for (key, value) in source {
            let exists = target.contains_key(key);

            match self.config.injection_type {
                InjectionType::Insert => {
                    if exists {
                        match self.config.strategy {
                            InjectionStrategy::FailOnConflict => result.add_failed(key.clone()),
                            InjectionStrategy::SkipOnConflict => result.add_skipped(key.clone()),
                            InjectionStrategy::OverwriteOnConflict => {
                                if !self.config.dry_run {
                                    target.insert(key.clone(), value.clone());
                                }
                                result.add_updated(key.clone());
                            }
                            InjectionStrategy::MergeOnConflict => {
                                result.add_skipped(key.clone());
                            }
                        }
                    } else if !self.config.dry_run {
                        target.insert(key.clone(), value.clone());
                        result.add_inserted(key.clone());
                    } else {
                        result.add_inserted(key.clone());
                    }
                }
                InjectionType::Update => {
                    if exists {
                        if !self.config.dry_run {
                            target.insert(key.clone(), value.clone());
                        }
                        result.add_updated(key.clone());
                    } else {
                        result.add_skipped(key.clone());
                    }
                }
                InjectionType::Upsert => {
                    if !self.config.dry_run {
                        target.insert(key.clone(), value.clone());
                    }
                    if exists {
                        result.add_updated(key.clone());
                    } else {
                        result.add_inserted(key.clone());
                    }
                }
                InjectionType::Replace => {
                    if !self.config.dry_run {
                        target.insert(key.clone(), value.clone());
                    }
                    result.add_updated(key.clone());
                }
                InjectionType::Append => {
                    if exists {
                        if !self.config.dry_run {
                            let existing = target.get(key).cloned().unwrap_or_default();
                            target.insert(key.clone(), format!("{}{}", existing, value));
                        }
                        result.add_updated(key.clone());
                    } else {
                        if !self.config.dry_run {
                            target.insert(key.clone(), value.clone());
                        }
                        result.add_inserted(key.clone());
                    }
                }
            }
        }

        self.stats.record(
            self.config.injection_type,
            result.inserted.len(),
            result.updated.len(),
            result.skipped.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[InjectionResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &InjectorStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings injector registry
#[derive(Debug, Clone, Default)]
pub struct SettingsInjectorRegistry {
    /// Injectors by ID
    injectors: HashMap<String, SettingsInjector>,
}

impl SettingsInjectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register injector
    pub fn register(&mut self, id: impl Into<String>, injector: SettingsInjector) {
        self.injectors.insert(id.into(), injector);
    }

    /// Unregister injector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.injectors.remove(id).is_some()
    }

    /// Get injector
    pub fn get(&self, id: &str) -> Option<&SettingsInjector> {
        self.injectors.get(id)
    }

    /// Get injector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsInjector> {
        self.injectors.get_mut(id)
    }

    /// Injector count
    pub fn count(&self) -> usize {
        self.injectors.len()
    }
}

/// Format injector registry
pub fn format_injector_registry(registry: &SettingsInjectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Injector Registry:\n");
    output.push_str(&format!("  Injectors: {}\n", registry.count()));
    output
}

/// Check if query is about injector
pub fn is_injector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("injector") || lower.contains("inject settings") || lower.contains("insert settings")
}

/// Fun fact about injector
pub fn injector_fun_fact() -> &'static str {
    "Anna's settings injectors insert configs into any target!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_type_display() {
        assert_eq!(format!("{}", InjectionType::Insert), "insert");
        assert_eq!(format!("{}", InjectionType::Upsert), "upsert");
    }

    #[test]
    fn test_injection_strategy_display() {
        assert_eq!(format!("{}", InjectionStrategy::FailOnConflict), "fail_on_conflict");
        assert_eq!(format!("{}", InjectionStrategy::SkipOnConflict), "skip_on_conflict");
    }

    #[test]
    fn test_config_new() {
        let c = InjectorConfig::new(InjectionType::Insert);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = InjectorConfig::new(InjectionType::Upsert)
            .strategy(InjectionStrategy::OverwriteOnConflict)
            .dry_run(true);
        assert_eq!(c.strategy, InjectionStrategy::OverwriteOnConflict);
        assert!(c.dry_run);
    }

    #[test]
    fn test_result_new() {
        let r = InjectionResult::new(InjectionType::Insert);
        assert_eq!(r.total_affected(), 0);
    }

    #[test]
    fn test_result_add() {
        let mut r = InjectionResult::new(InjectionType::Insert);
        r.add_inserted("key1".to_string());
        r.add_updated("key2".to_string());
        assert_eq!(r.total_affected(), 2);
    }

    #[test]
    fn test_stats_record() {
        let mut s = InjectorStats::default();
        s.record(InjectionType::Insert, 5, 3, 2);
        assert_eq!(s.total_injections, 1);
        assert_eq!(s.total_inserted, 5);
    }

    #[test]
    fn test_injector_new() {
        let i = SettingsInjector::new(InjectorConfig::new(InjectionType::Insert));
        assert_eq!(i.result_count(), 0);
    }

    #[test]
    fn test_injector_inject_insert() {
        let mut i = SettingsInjector::new(InjectorConfig::new(InjectionType::Insert));
        let mut target = HashMap::new();
        let mut source = HashMap::new();
        source.insert("key".to_string(), "value".to_string());

        let r = i.inject(&mut target, &source);
        assert_eq!(r.inserted.len(), 1);
        assert_eq!(target.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_injector_inject_upsert() {
        let mut i = SettingsInjector::new(InjectorConfig::new(InjectionType::Upsert));
        let mut target = HashMap::new();
        target.insert("existing".to_string(), "old".to_string());

        let mut source = HashMap::new();
        source.insert("existing".to_string(), "new".to_string());
        source.insert("new_key".to_string(), "value".to_string());

        let r = i.inject(&mut target, &source);
        assert_eq!(r.updated.len(), 1);
        assert_eq!(r.inserted.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsInjectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsInjectorRegistry::new();
        r.register("inj1", SettingsInjector::new(InjectorConfig::new(InjectionType::Insert)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_injector_query() {
        assert!(is_injector_query("settings injector"));
        assert!(!is_injector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = injector_fun_fact();
        assert!(fact.contains("injector"));
    }
}
