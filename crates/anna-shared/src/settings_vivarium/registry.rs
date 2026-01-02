use std::collections::HashMap;
use super::vivarium::SettingsVivarium;

/// Vivarium registry
#[derive(Debug, Clone, Default)]
pub struct VivariumRegistry {
    /// Vivariums by ID
    vivariums: HashMap<String, SettingsVivarium>,
}

impl VivariumRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register vivarium
    pub fn register(&mut self, id: impl Into<String>, vivarium: SettingsVivarium) {
        self.vivariums.insert(id.into(), vivarium);
    }

    /// Unregister vivarium
    pub fn unregister(&mut self, id: &str) -> bool {
        self.vivariums.remove(id).is_some()
    }

    /// Get vivarium
    pub fn get(&self, id: &str) -> Option<&SettingsVivarium> {
        self.vivariums.get(id)
    }

    /// Get vivarium mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsVivarium> {
        self.vivariums.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.vivariums.len()
    }
}

/// Format vivarium registry
pub fn format_vivarium_registry(registry: &VivariumRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Vivarium Registry:\n");
    output.push_str(&format!("  Vivariums: {}\n", registry.count()));
    output
}

/// Check if query is about vivarium
pub fn is_vivarium_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings vivarium") || lower.contains("vivarium settings") || lower.contains("living vivarium")
}

/// Fun fact about vivarium
pub fn vivarium_fun_fact() -> &'static str {
    "Anna's settings vivarium maintains animal habitat boundaries!"
}
