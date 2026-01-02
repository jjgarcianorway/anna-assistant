// v0.0.690: Settings Combiner Registry (Phase 266)
// Registry for managing multiple combiners

use std::collections::HashMap;
use crate::settings_combiner::combiner::SettingsCombiner;

/// Combiner registry
#[derive(Debug, Clone, Default)]
pub struct CombinerRegistry {
    /// Combiners by ID
    combiners: HashMap<String, SettingsCombiner>,
}

impl CombinerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register combiner
    pub fn register(&mut self, id: impl Into<String>, combiner: SettingsCombiner) {
        self.combiners.insert(id.into(), combiner);
    }

    /// Unregister combiner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.combiners.remove(id).is_some()
    }

    /// Get combiner
    pub fn get(&self, id: &str) -> Option<&SettingsCombiner> {
        self.combiners.get(id)
    }

    /// Get combiner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCombiner> {
        self.combiners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.combiners.len()
    }
}

/// Format combiner registry
pub fn format_combiner_registry(registry: &CombinerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Combiner Registry:\n");
    output.push_str(&format!("  Combiners: {}\n", registry.count()));
    output
}
