// v0.0.710: Settings Brief - Registry (Phase 286)
// Registry for managing multiple briefs

use std::collections::HashMap;

use super::brief::SettingsBrief;

/// Brief registry
#[derive(Debug, Clone, Default)]
pub struct BriefRegistry {
    /// Briefs by ID
    briefs: HashMap<String, SettingsBrief>,
}

impl BriefRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register brief
    pub fn register(&mut self, id: impl Into<String>, brief: SettingsBrief) {
        self.briefs.insert(id.into(), brief);
    }

    /// Unregister brief
    pub fn unregister(&mut self, id: &str) -> bool {
        self.briefs.remove(id).is_some()
    }

    /// Get brief
    pub fn get(&self, id: &str) -> Option<&SettingsBrief> {
        self.briefs.get(id)
    }

    /// Get brief mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBrief> {
        self.briefs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.briefs.len()
    }
}
