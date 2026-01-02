// v0.0.643: Sanitizer Registry (Phase 219)
// Registry for managing multiple sanitizers

use std::collections::HashMap;

use super::sanitizer::SettingsSanitizer;

/// Settings sanitizer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsSanitizerRegistry {
    /// Sanitizers by ID
    sanitizers: HashMap<String, SettingsSanitizer>,
}

impl SettingsSanitizerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sanitizer
    pub fn register(&mut self, id: impl Into<String>, sanitizer: SettingsSanitizer) {
        self.sanitizers.insert(id.into(), sanitizer);
    }

    /// Unregister sanitizer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.sanitizers.remove(id).is_some()
    }

    /// Get sanitizer
    pub fn get(&self, id: &str) -> Option<&SettingsSanitizer> {
        self.sanitizers.get(id)
    }

    /// Get sanitizer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSanitizer> {
        self.sanitizers.get_mut(id)
    }

    /// Sanitizer count
    pub fn count(&self) -> usize {
        self.sanitizers.len()
    }
}

/// Format sanitizer registry
pub fn format_sanitizer_registry(registry: &SettingsSanitizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sanitizer Registry:\n");
    output.push_str(&format!("  Sanitizers: {}\n", registry.count()));
    output
}

/// Check if query is about sanitizer
pub fn is_sanitizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("sanitizer") || lower.contains("sanitize settings") || lower.contains("clean")
}

/// Fun fact about sanitizer
pub fn sanitizer_fun_fact() -> &'static str {
    "Anna's settings sanitizers clean and normalize values!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::SanitizerConfig;
    use super::super::types::SanitizationType;

    #[test]
    fn test_registry_new() {
        let r = SettingsSanitizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsSanitizerRegistry::new();
        r.register("san1", SettingsSanitizer::new(SanitizerConfig::new(SanitizationType::Trim)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_sanitizer_query() {
        assert!(is_sanitizer_query("settings sanitizer"));
        assert!(!is_sanitizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = sanitizer_fun_fact();
        assert!(fact.contains("sanitizer"));
    }
}
