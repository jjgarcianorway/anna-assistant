// v0.0.652: Settings Binder - Binder
// Main binder and registry

use std::collections::HashMap;
use super::binding::{BindingDef, BindingResult};
use super::config::BinderConfig;
use super::stats::BinderStats;
use super::types::BindingState;

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
