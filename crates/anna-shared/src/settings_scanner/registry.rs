// v0.0.684: Settings Scanner - Registry
// Registry for managing multiple scanners

use super::scanner::SettingsScanner;
use std::collections::HashMap;

/// Scanner registry
#[derive(Debug, Clone, Default)]
pub struct ScannerRegistry {
    /// Scanners by ID
    scanners: HashMap<String, SettingsScanner>,
}

impl ScannerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register scanner
    pub fn register(&mut self, id: impl Into<String>, scanner: SettingsScanner) {
        self.scanners.insert(id.into(), scanner);
    }

    /// Unregister scanner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.scanners.remove(id).is_some()
    }

    /// Get scanner
    pub fn get(&self, id: &str) -> Option<&SettingsScanner> {
        self.scanners.get(id)
    }

    /// Get scanner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsScanner> {
        self.scanners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.scanners.len()
    }
}

/// Format scanner registry
pub fn format_scanner_registry(registry: &ScannerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Scanner Registry:\n");
    output.push_str(&format!("  Scanners: {}\n", registry.count()));
    output
}
