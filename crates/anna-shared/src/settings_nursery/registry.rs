// v0.0.769: Settings Nursery - Registry (Phase 345)

use std::collections::HashMap;
use super::nursery::SettingsNursery;

/// Nursery registry
#[derive(Debug, Clone, Default)]
pub struct NurseryRegistry {
    /// Nurseries by ID
    nurseries: HashMap<String, SettingsNursery>,
}

impl NurseryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register nursery
    pub fn register(&mut self, id: impl Into<String>, nursery: SettingsNursery) {
        self.nurseries.insert(id.into(), nursery);
    }

    /// Unregister nursery
    pub fn unregister(&mut self, id: &str) -> bool {
        self.nurseries.remove(id).is_some()
    }

    /// Get nursery
    pub fn get(&self, id: &str) -> Option<&SettingsNursery> {
        self.nurseries.get(id)
    }

    /// Get nursery mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNursery> {
        self.nurseries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.nurseries.len()
    }
}

/// Format nursery registry
pub fn format_nursery_registry(registry: &NurseryRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Nursery Registry:\n");
    output.push_str(&format!("  Nurseries: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::NurseryConfig;

    #[test]
    fn test_registry_new() {
        let r = NurseryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NurseryRegistry::new();
        r.register("n1", SettingsNursery::new(NurseryConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
