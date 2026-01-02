// v0.0.751: Settings Borough Registry
// Registry for managing multiple boroughs

use super::borough::SettingsBorough;
use std::collections::HashMap;

/// Borough registry
#[derive(Debug, Clone, Default)]
pub struct BoroughRegistry {
    /// Boroughs by ID
    boroughs: HashMap<String, SettingsBorough>,
}

impl BoroughRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register borough
    pub fn register(&mut self, id: impl Into<String>, borough: SettingsBorough) {
        self.boroughs.insert(id.into(), borough);
    }

    /// Unregister borough
    pub fn unregister(&mut self, id: &str) -> bool {
        self.boroughs.remove(id).is_some()
    }

    /// Get borough
    pub fn get(&self, id: &str) -> Option<&SettingsBorough> {
        self.boroughs.get(id)
    }

    /// Get borough mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBorough> {
        self.boroughs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.boroughs.len()
    }
}
