// v0.0.764: Settings Pasture - Registry (Phase 340)

use std::collections::HashMap;

use super::pasture::SettingsPasture;

/// Pasture registry
#[derive(Debug, Clone, Default)]
pub struct PastureRegistry {
    /// Pastures by ID
    pastures: HashMap<String, SettingsPasture>,
}

impl PastureRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register pasture
    pub fn register(&mut self, id: impl Into<String>, pasture: SettingsPasture) {
        self.pastures.insert(id.into(), pasture);
    }

    /// Unregister pasture
    pub fn unregister(&mut self, id: &str) -> bool {
        self.pastures.remove(id).is_some()
    }

    /// Get pasture
    pub fn get(&self, id: &str) -> Option<&SettingsPasture> {
        self.pastures.get(id)
    }

    /// Get pasture mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPasture> {
        self.pastures.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.pastures.len()
    }
}

/// Format pasture registry
pub fn format_pasture_registry(registry: &PastureRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Pasture Registry:\n");
    output.push_str(&format!("  Pastures: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_pasture::config::PastureConfig;

    #[test]
    fn test_registry_new() {
        let r = PastureRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PastureRegistry::new();
        r.register("p1", SettingsPasture::new(PastureConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
