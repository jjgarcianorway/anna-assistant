// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Registry

use std::collections::HashMap;
use super::union::SettingsUnion;

/// Union registry
#[derive(Debug, Clone, Default)]
pub struct UnionRegistry {
    /// Unions by ID
    unions: HashMap<String, SettingsUnion>,
}

impl UnionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register union
    pub fn register(&mut self, id: impl Into<String>, union: SettingsUnion) {
        self.unions.insert(id.into(), union);
    }

    /// Unregister union
    pub fn unregister(&mut self, id: &str) -> bool {
        self.unions.remove(id).is_some()
    }

    /// Get union
    pub fn get(&self, id: &str) -> Option<&SettingsUnion> {
        self.unions.get(id)
    }

    /// Get union mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsUnion> {
        self.unions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.unions.len()
    }
}

/// Format union registry
pub fn format_union_registry(registry: &UnionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Union Registry:\n");
    output.push_str(&format!("  Unions: {}\n", registry.count()));
    output
}

/// Check if query is about union
pub fn is_union_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings union") || lower.contains("union settings") || lower.contains("political union")
}

/// Fun fact about union
pub fn union_fun_fact() -> &'static str {
    "Anna's settings union establishes political integration!"
}
