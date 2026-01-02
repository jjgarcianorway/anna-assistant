// v0.0.762: Settings Field Registry (Phase 338)
// Field registry management

use std::collections::HashMap;
use super::field::SettingsField;

/// Field registry
#[derive(Debug, Clone, Default)]
pub struct FieldRegistry {
    /// Fields by ID
    fields: HashMap<String, SettingsField>,
}

impl FieldRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register field
    pub fn register(&mut self, id: impl Into<String>, field: SettingsField) {
        self.fields.insert(id.into(), field);
    }

    /// Unregister field
    pub fn unregister(&mut self, id: &str) -> bool {
        self.fields.remove(id).is_some()
    }

    /// Get field
    pub fn get(&self, id: &str) -> Option<&SettingsField> {
        self.fields.get(id)
    }

    /// Get field mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsField> {
        self.fields.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.fields.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_field::config::FieldConfig;

    #[test]
    fn test_registry_new() {
        let r = FieldRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FieldRegistry::new();
        r.register("f1", SettingsField::new(FieldConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
