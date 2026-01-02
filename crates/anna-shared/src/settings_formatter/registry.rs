// v0.0.644: Settings Formatter Registry (Phase 220)
// Registry for managing multiple formatters

use std::collections::HashMap;

use super::formatter::SettingsFormatter;

/// Settings formatter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsFormatterRegistry {
    /// Formatters by ID
    formatters: HashMap<String, SettingsFormatter>,
}

impl SettingsFormatterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register formatter
    pub fn register(&mut self, id: impl Into<String>, formatter: SettingsFormatter) {
        self.formatters.insert(id.into(), formatter);
    }

    /// Unregister formatter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.formatters.remove(id).is_some()
    }

    /// Get formatter
    pub fn get(&self, id: &str) -> Option<&SettingsFormatter> {
        self.formatters.get(id)
    }

    /// Get formatter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFormatter> {
        self.formatters.get_mut(id)
    }

    /// Formatter count
    pub fn count(&self) -> usize {
        self.formatters.len()
    }
}

/// Format formatter registry
pub fn format_formatter_registry(registry: &SettingsFormatterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Formatter Registry:\n");
    output.push_str(&format!("  Formatters: {}\n", registry.count()));
    output
}

/// Check if query is about formatter
pub fn is_formatter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("formatter") || lower.contains("format settings") || lower.contains("output format")
}

/// Fun fact about formatter
pub fn formatter_fun_fact() -> &'static str {
    "Anna's settings formatters convert values to any output format!"
}
