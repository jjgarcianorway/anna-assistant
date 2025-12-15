// v0.0.652: Settings Binder (Phase 228)
// Binder for connecting settings to runtime objects

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Binding type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BindingType {
    /// One-way binding
    #[default]
    OneWay,
    /// Two-way binding
    TwoWay,
    /// One-time binding
    OneTime,
    /// Lazy binding
    Lazy,
    /// Eager binding
    Eager,
}

impl std::fmt::Display for BindingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OneWay => write!(f, "one_way"),
            Self::TwoWay => write!(f, "two_way"),
            Self::OneTime => write!(f, "one_time"),
            Self::Lazy => write!(f, "lazy"),
            Self::Eager => write!(f, "eager"),
        }
    }
}

/// Binding state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BindingState {
    /// Unbound
    #[default]
    Unbound,
    /// Bound
    Bound,
    /// Pending
    Pending,
    /// Error
    Error,
}

impl std::fmt::Display for BindingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unbound => write!(f, "unbound"),
            Self::Bound => write!(f, "bound"),
            Self::Pending => write!(f, "pending"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Binding definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingDef {
    /// Source key
    pub source: String,
    /// Target path
    pub target: String,
    /// Binding type
    pub binding_type: BindingType,
    /// Current state
    pub state: BindingState,
}

impl BindingDef {
    /// Create new binding
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            binding_type: BindingType::OneWay,
            state: BindingState::Unbound,
        }
    }

    /// Set binding type
    pub fn binding_type(mut self, binding_type: BindingType) -> Self {
        self.binding_type = binding_type;
        self
    }

    /// Is bound
    pub fn is_bound(&self) -> bool {
        self.state == BindingState::Bound
    }
}

/// Binder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderConfig {
    /// Default binding type
    pub default_type: BindingType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Auto-bind
    pub auto_bind: bool,
    /// Validate on bind
    pub validate_on_bind: bool,
}

impl BinderConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            default_type: BindingType::OneWay,
            category: None,
            auto_bind: false,
            validate_on_bind: true,
        }
    }

    /// Set default type
    pub fn default_type(mut self, binding_type: BindingType) -> Self {
        self.default_type = binding_type;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set auto-bind
    pub fn auto_bind(mut self, auto: bool) -> Self {
        self.auto_bind = auto;
        self
    }

    /// Set validate on bind
    pub fn validate_on_bind(mut self, validate: bool) -> Self {
        self.validate_on_bind = validate;
        self
    }
}

impl Default for BinderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Binding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingResult {
    /// Source key
    pub source: String,
    /// Target path
    pub target: String,
    /// Was successful
    pub success: bool,
    /// State after binding
    pub state: BindingState,
    /// Error message if failed
    pub error: Option<String>,
}

impl BindingResult {
    /// Create success result
    pub fn success(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            success: true,
            state: BindingState::Bound,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(source: impl Into<String>, target: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            success: false,
            state: BindingState::Error,
            error: Some(error.into()),
        }
    }
}

/// Binder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinderStats {
    /// Total bind attempts
    pub total_binds: usize,
    /// Successful binds
    pub successful: usize,
    /// Failed binds
    pub failed: usize,
    /// By binding type
    pub by_type: HashMap<String, usize>,
}

impl BinderStats {
    /// Record bind
    pub fn record(&mut self, binding_type: BindingType, success: bool) {
        self.total_binds += 1;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        *self.by_type.entry(binding_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_binds == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_binds as f64
        }
    }
}

/// Settings binder
#[derive(Debug, Clone, Default)]
pub struct SettingsBinder {
    /// Config
    config: BinderConfig,
    /// Bindings
    bindings: Vec<BindingDef>,
    /// Results
    results: Vec<BindingResult>,
    /// Stats
    stats: BinderStats,
}

impl SettingsBinder {
    /// Create new binder
    pub fn new(config: BinderConfig) -> Self {
        Self {
            config,
            bindings: Vec::new(),
            results: Vec::new(),
            stats: BinderStats::default(),
        }
    }

    /// Add binding
    pub fn add_binding(&mut self, binding: BindingDef) {
        self.bindings.push(binding);
    }

    /// Bind all
    pub fn bind_all(&mut self) -> Vec<BindingResult> {
        let mut results = Vec::new();

        for binding in &mut self.bindings {
            let result = if binding.source.is_empty() || binding.target.is_empty() {
                BindingResult::failure(&binding.source, &binding.target, "Empty source or target")
            } else {
                binding.state = BindingState::Bound;
                BindingResult::success(&binding.source, &binding.target)
            };

            self.stats.record(binding.binding_type, result.success);
            results.push(result);
        }

        self.results.extend(results.clone());
        results
    }

    /// Unbind all
    pub fn unbind_all(&mut self) {
        for binding in &mut self.bindings {
            binding.state = BindingState::Unbound;
        }
    }

    /// Get bindings
    pub fn bindings(&self) -> &[BindingDef] {
        &self.bindings
    }

    /// Get results
    pub fn results(&self) -> &[BindingResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &BinderStats {
        &self.stats
    }

    /// Binding count
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Bound count
    pub fn bound_count(&self) -> usize {
        self.bindings.iter().filter(|b| b.is_bound()).count()
    }

    /// Clear bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.results.clear();
    }
}

/// Settings binder registry
#[derive(Debug, Clone, Default)]
pub struct SettingsBinderRegistry {
    /// Binders by ID
    binders: HashMap<String, SettingsBinder>,
}

impl SettingsBinderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register binder
    pub fn register(&mut self, id: impl Into<String>, binder: SettingsBinder) {
        self.binders.insert(id.into(), binder);
    }

    /// Unregister binder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.binders.remove(id).is_some()
    }

    /// Get binder
    pub fn get(&self, id: &str) -> Option<&SettingsBinder> {
        self.binders.get(id)
    }

    /// Get binder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBinder> {
        self.binders.get_mut(id)
    }

    /// Binder count
    pub fn count(&self) -> usize {
        self.binders.len()
    }
}

/// Format binder registry
pub fn format_binder_registry(registry: &SettingsBinderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Binder Registry:\n");
    output.push_str(&format!("  Binders: {}\n", registry.count()));
    output
}

/// Check if query is about binder
pub fn is_binder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("binder") || lower.contains("bind settings") || lower.contains("settings binding")
}

/// Fun fact about binder
pub fn binder_fun_fact() -> &'static str {
    "Anna's settings binders connect configs to runtime objects!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_type_display() {
        assert_eq!(format!("{}", BindingType::OneWay), "one_way");
        assert_eq!(format!("{}", BindingType::TwoWay), "two_way");
    }

    #[test]
    fn test_binding_state_display() {
        assert_eq!(format!("{}", BindingState::Bound), "bound");
        assert_eq!(format!("{}", BindingState::Unbound), "unbound");
    }

    #[test]
    fn test_binding_def_new() {
        let b = BindingDef::new("src", "dst");
        assert!(!b.is_bound());
    }

    #[test]
    fn test_config_new() {
        let c = BinderConfig::new();
        assert!(c.validate_on_bind);
    }

    #[test]
    fn test_config_builder() {
        let c = BinderConfig::new()
            .default_type(BindingType::TwoWay)
            .auto_bind(true);
        assert_eq!(c.default_type, BindingType::TwoWay);
        assert!(c.auto_bind);
    }

    #[test]
    fn test_result_success() {
        let r = BindingResult::success("src", "dst");
        assert!(r.success);
        assert_eq!(r.state, BindingState::Bound);
    }

    #[test]
    fn test_result_failure() {
        let r = BindingResult::failure("src", "dst", "error");
        assert!(!r.success);
        assert!(r.error.is_some());
    }

    #[test]
    fn test_stats_record() {
        let mut s = BinderStats::default();
        s.record(BindingType::OneWay, true);
        s.record(BindingType::TwoWay, false);
        assert_eq!(s.total_binds, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_binder_new() {
        let b = SettingsBinder::new(BinderConfig::new());
        assert_eq!(b.binding_count(), 0);
    }

    #[test]
    fn test_binder_add_binding() {
        let mut b = SettingsBinder::new(BinderConfig::new());
        b.add_binding(BindingDef::new("src", "dst"));
        assert_eq!(b.binding_count(), 1);
    }

    #[test]
    fn test_binder_bind_all() {
        let mut b = SettingsBinder::new(BinderConfig::new());
        b.add_binding(BindingDef::new("src", "dst"));
        let results = b.bind_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsBinderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsBinderRegistry::new();
        r.register("binder1", SettingsBinder::new(BinderConfig::new()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_binder_query() {
        assert!(is_binder_query("settings binder"));
        assert!(!is_binder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = binder_fun_fact();
        assert!(fact.contains("binder"));
    }
}
