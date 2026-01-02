// v0.0.756: Settings Lot Registry (Phase 332)
// Lot registry and utility functions

use std::collections::HashMap;
use super::lot::SettingsLot;

/// Lot registry
#[derive(Debug, Clone, Default)]
pub struct LotRegistry {
    /// Lots by ID
    lots: HashMap<String, SettingsLot>,
}

impl LotRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register lot
    pub fn register(&mut self, id: impl Into<String>, lot: SettingsLot) {
        self.lots.insert(id.into(), lot);
    }

    /// Unregister lot
    pub fn unregister(&mut self, id: &str) -> bool {
        self.lots.remove(id).is_some()
    }

    /// Get lot
    pub fn get(&self, id: &str) -> Option<&SettingsLot> {
        self.lots.get(id)
    }

    /// Get lot mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsLot> {
        self.lots.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.lots.len()
    }
}

/// Format lot registry
pub fn format_lot_registry(registry: &LotRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Lot Registry:\n");
    output.push_str(&format!("  Lots: {}\n", registry.count()));
    output
}

/// Check if query is about lot
pub fn is_lot_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings lot") || lower.contains("lot settings") || lower.contains("land lot")
}

/// Fun fact about lot
pub fn lot_fun_fact() -> &'static str {
    "Anna's settings lot establishes property boundaries!"
}
