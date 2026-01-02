// v0.0.746: Settings Province - Registry (Phase 322)
// Province registry

use std::collections::HashMap;
use super::province::SettingsProvince;

/// Province registry
#[derive(Debug, Clone, Default)]
pub struct ProvinceRegistry {
    /// Provinces by ID
    provinces: HashMap<String, SettingsProvince>,
}

impl ProvinceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register province
    pub fn register(&mut self, id: impl Into<String>, province: SettingsProvince) {
        self.provinces.insert(id.into(), province);
    }

    /// Unregister province
    pub fn unregister(&mut self, id: &str) -> bool {
        self.provinces.remove(id).is_some()
    }

    /// Get province
    pub fn get(&self, id: &str) -> Option<&SettingsProvince> {
        self.provinces.get(id)
    }

    /// Get province mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProvince> {
        self.provinces.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.provinces.len()
    }
}

/// Format province registry
pub fn format_province_registry(registry: &ProvinceRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Province Registry:\n");
    output.push_str(&format!("  Provinces: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_province::ProvinceConfig;

    #[test]
    fn test_registry_new() {
        let r = ProvinceRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ProvinceRegistry::new();
        r.register("p1", SettingsProvince::new(ProvinceConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
