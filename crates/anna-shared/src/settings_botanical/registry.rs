// v0.0.773: Settings Botanical Registry (Phase 349)
// Registry for managing multiple botanical systems

use std::collections::HashMap;
use super::botanical::SettingsBotanical;

/// Botanical registry
#[derive(Debug, Clone, Default)]
pub struct BotanicalRegistry {
    /// Botanicals by ID
    botanicals: HashMap<String, SettingsBotanical>,
}

impl BotanicalRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register botanical
    pub fn register(&mut self, id: impl Into<String>, botanical: SettingsBotanical) {
        self.botanicals.insert(id.into(), botanical);
    }

    /// Unregister botanical
    pub fn unregister(&mut self, id: &str) -> bool {
        self.botanicals.remove(id).is_some()
    }

    /// Get botanical
    pub fn get(&self, id: &str) -> Option<&SettingsBotanical> {
        self.botanicals.get(id)
    }

    /// Get botanical mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBotanical> {
        self.botanicals.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.botanicals.len()
    }
}

/// Format botanical registry
pub fn format_botanical_registry(registry: &BotanicalRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Botanical Registry:\n");
    output.push_str(&format!("  Botanicals: {}\n", registry.count()));
    output
}
