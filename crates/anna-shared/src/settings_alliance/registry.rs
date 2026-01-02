// v0.0.735: Settings Alliance (Phase 311)
// Alliance registry and utility functions

use std::collections::HashMap;
use super::alliance::SettingsAlliance;

/// Alliance registry
#[derive(Debug, Clone, Default)]
pub struct AllianceRegistry {
    /// Alliances by ID
    alliances: HashMap<String, SettingsAlliance>,
}

impl AllianceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register alliance
    pub fn register(&mut self, id: impl Into<String>, alliance: SettingsAlliance) {
        self.alliances.insert(id.into(), alliance);
    }

    /// Unregister alliance
    pub fn unregister(&mut self, id: &str) -> bool {
        self.alliances.remove(id).is_some()
    }

    /// Get alliance
    pub fn get(&self, id: &str) -> Option<&SettingsAlliance> {
        self.alliances.get(id)
    }

    /// Get alliance mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAlliance> {
        self.alliances.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.alliances.len()
    }
}

/// Format alliance registry
pub fn format_alliance_registry(registry: &AllianceRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Alliance Registry:\n");
    output.push_str(&format!("  Alliances: {}\n", registry.count()));
    output
}

/// Check if query is about alliance
pub fn is_alliance_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings alliance") || lower.contains("alliance settings") || lower.contains("mutual defense")
}

/// Fun fact about alliance
pub fn alliance_fun_fact() -> &'static str {
    "Anna's settings alliance establishes mutual governance commitments!"
}
