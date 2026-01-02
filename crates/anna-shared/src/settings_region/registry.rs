// v0.0.747: Settings Region Registry (Phase 323)
// Region registry management

use std::collections::HashMap;
use super::region::SettingsRegion;

/// Region registry
#[derive(Debug, Clone, Default)]
pub struct RegionRegistry {
    /// Regions by ID
    regions: HashMap<String, SettingsRegion>,
}

impl RegionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register region
    pub fn register(&mut self, id: impl Into<String>, region: SettingsRegion) {
        self.regions.insert(id.into(), region);
    }

    /// Unregister region
    pub fn unregister(&mut self, id: &str) -> bool {
        self.regions.remove(id).is_some()
    }

    /// Get region
    pub fn get(&self, id: &str) -> Option<&SettingsRegion> {
        self.regions.get(id)
    }

    /// Get region mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRegion> {
        self.regions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.regions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::RegionConfig;

    #[test]
    fn test_registry_new() {
        let r = RegionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = RegionRegistry::new();
        r.register("r1", SettingsRegion::new(RegionConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
