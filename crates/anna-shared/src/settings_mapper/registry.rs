// v0.0.651: Settings Mapper Registry (Phase 227)
// Registry for managing multiple mappers

use std::collections::HashMap;

use crate::settings_mapper::mapper::SettingsMapper;

/// Settings mapper registry
#[derive(Debug, Clone, Default)]
pub struct SettingsMapperRegistry {
    /// Mappers by ID
    mappers: HashMap<String, SettingsMapper>,
}

impl SettingsMapperRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register mapper
    pub fn register(&mut self, id: impl Into<String>, mapper: SettingsMapper) {
        self.mappers.insert(id.into(), mapper);
    }

    /// Unregister mapper
    pub fn unregister(&mut self, id: &str) -> bool {
        self.mappers.remove(id).is_some()
    }

    /// Get mapper
    pub fn get(&self, id: &str) -> Option<&SettingsMapper> {
        self.mappers.get(id)
    }

    /// Get mapper mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMapper> {
        self.mappers.get_mut(id)
    }

    /// Mapper count
    pub fn count(&self) -> usize {
        self.mappers.len()
    }
}

/// Format mapper registry
pub fn format_mapper_registry(registry: &SettingsMapperRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Mapper Registry:\n");
    output.push_str(&format!("  Mappers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_mapper::types::MapperConfig;

    #[test]
    fn test_registry_new() {
        let r = SettingsMapperRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsMapperRegistry::new();
        r.register("map1", SettingsMapper::new(MapperConfig::new()));
        assert_eq!(r.count(), 1);
    }
}
