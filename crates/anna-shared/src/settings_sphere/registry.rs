// v0.0.741: Settings Sphere Registry (Phase 317)
// Registry for managing multiple spheres

use super::sphere::SettingsSphere;
use std::collections::HashMap;

/// Sphere registry
#[derive(Debug, Clone, Default)]
pub struct SphereRegistry {
    /// Spheres by ID
    spheres: HashMap<String, SettingsSphere>,
}

impl SphereRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sphere
    pub fn register(&mut self, id: impl Into<String>, sphere: SettingsSphere) {
        self.spheres.insert(id.into(), sphere);
    }

    /// Unregister sphere
    pub fn unregister(&mut self, id: &str) -> bool {
        self.spheres.remove(id).is_some()
    }

    /// Get sphere
    pub fn get(&self, id: &str) -> Option<&SettingsSphere> {
        self.spheres.get(id)
    }

    /// Get sphere mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSphere> {
        self.spheres.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.spheres.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_sphere::types::SphereConfig;

    #[test]
    fn test_registry_new() {
        let r = SphereRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SphereRegistry::new();
        r.register("s1", SettingsSphere::new(SphereConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
