// v0.0.731: Settings Pact (Phase 307)
// Pact registry

use std::collections::HashMap;

use super::pact::SettingsPact;

/// Pact registry
#[derive(Debug, Clone, Default)]
pub struct PactRegistry {
    /// Pacts by ID
    pacts: HashMap<String, SettingsPact>,
}

impl PactRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register pact
    pub fn register(&mut self, id: impl Into<String>, pact: SettingsPact) {
        self.pacts.insert(id.into(), pact);
    }

    /// Unregister pact
    pub fn unregister(&mut self, id: &str) -> bool {
        self.pacts.remove(id).is_some()
    }

    /// Get pact
    pub fn get(&self, id: &str) -> Option<&SettingsPact> {
        self.pacts.get(id)
    }

    /// Get pact mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPact> {
        self.pacts.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.pacts.len()
    }
}

/// Format pact registry
pub fn format_pact_registry(registry: &PactRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Pact Registry:\n");
    output.push_str(&format!("  Pacts: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_pact::structs::PactConfig;

    #[test]
    fn test_registry_new() {
        let r = PactRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PactRegistry::new();
        r.register("p1", SettingsPact::new(PactConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
