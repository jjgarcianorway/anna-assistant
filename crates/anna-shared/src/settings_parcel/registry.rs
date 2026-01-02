// v0.0.757: Settings Parcel - Registry (Phase 333)

use std::collections::HashMap;
use super::parcel::SettingsParcel;

/// Parcel registry
#[derive(Debug, Clone, Default)]
pub struct ParcelRegistry {
    /// Parcels by ID
    parcels: HashMap<String, SettingsParcel>,
}

impl ParcelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register parcel
    pub fn register(&mut self, id: impl Into<String>, parcel: SettingsParcel) {
        self.parcels.insert(id.into(), parcel);
    }

    /// Unregister parcel
    pub fn unregister(&mut self, id: &str) -> bool {
        self.parcels.remove(id).is_some()
    }

    /// Get parcel
    pub fn get(&self, id: &str) -> Option<&SettingsParcel> {
        self.parcels.get(id)
    }

    /// Get parcel mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsParcel> {
        self.parcels.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.parcels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::ParcelConfig;

    #[test]
    fn test_registry_new() {
        let r = ParcelRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ParcelRegistry::new();
        r.register("p1", SettingsParcel::new(ParcelConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
