// v0.0.742: Zone Registry (Phase 318)

use std::collections::HashMap;
use super::zone::SettingsZone;

/// Zone registry
#[derive(Debug, Clone, Default)]
pub struct ZoneRegistry {
    /// Zones by ID
    zones: HashMap<String, SettingsZone>,
}

impl ZoneRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register zone
    pub fn register(&mut self, id: impl Into<String>, zone: SettingsZone) {
        self.zones.insert(id.into(), zone);
    }

    /// Unregister zone
    pub fn unregister(&mut self, id: &str) -> bool {
        self.zones.remove(id).is_some()
    }

    /// Get zone
    pub fn get(&self, id: &str) -> Option<&SettingsZone> {
        self.zones.get(id)
    }

    /// Get zone mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsZone> {
        self.zones.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.zones.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_zone::config::ZoneConfig;

    #[test]
    fn test_registry_new() {
        let r = ZoneRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ZoneRegistry::new();
        r.register("z1", SettingsZone::new(ZoneConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
