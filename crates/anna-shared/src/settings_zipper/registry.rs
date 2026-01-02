// v0.0.683: Zipper Registry
// Registry for managing multiple zipper instances

use std::collections::HashMap;
use super::zipper::SettingsZipper;

/// Zipper registry
#[derive(Debug, Clone, Default)]
pub struct ZipperRegistry {
    /// Zippers by ID
    zippers: HashMap<String, SettingsZipper>,
}

impl ZipperRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register zipper
    pub fn register(&mut self, id: impl Into<String>, zipper: SettingsZipper) {
        self.zippers.insert(id.into(), zipper);
    }

    /// Unregister zipper
    pub fn unregister(&mut self, id: &str) -> bool {
        self.zippers.remove(id).is_some()
    }

    /// Get zipper
    pub fn get(&self, id: &str) -> Option<&SettingsZipper> {
        self.zippers.get(id)
    }

    /// Get zipper mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsZipper> {
        self.zippers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.zippers.len()
    }
}

/// Format zipper registry
pub fn format_zipper_registry(registry: &ZipperRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Zipper Registry:\n");
    output.push_str(&format!("  Zippers: {}\n", registry.count()));
    output
}
