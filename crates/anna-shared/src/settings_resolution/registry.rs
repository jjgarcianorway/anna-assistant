// v0.0.664: Settings Resolver Registry
// Registry and utility functions

use std::collections::HashMap;
use super::resolver::SettingsResolver;

/// Settings resolver registry
#[derive(Debug, Clone, Default)]
pub struct SettingsResolverRegistry {
    /// Resolvers by ID
    resolvers: HashMap<String, SettingsResolver>,
}

impl SettingsResolverRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register resolver
    pub fn register(&mut self, id: impl Into<String>, resolver: SettingsResolver) {
        self.resolvers.insert(id.into(), resolver);
    }

    /// Unregister resolver
    pub fn unregister(&mut self, id: &str) -> bool {
        self.resolvers.remove(id).is_some()
    }

    /// Get resolver
    pub fn get(&self, id: &str) -> Option<&SettingsResolver> {
        self.resolvers.get(id)
    }

    /// Get resolver mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsResolver> {
        self.resolvers.get_mut(id)
    }

    /// Resolver count
    pub fn count(&self) -> usize {
        self.resolvers.len()
    }
}

/// Format resolver registry
pub fn format_resolver_registry(registry: &SettingsResolverRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Resolver Registry:\n");
    output.push_str(&format!("  Resolvers: {}\n", registry.count()));
    output
}

/// Check if query is about resolver
pub fn is_resolver_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("resolve") || lower.contains("settings resolver") || lower.contains("resolution")
}

/// Fun fact about resolver
pub fn resolver_fun_fact() -> &'static str {
    "Anna's settings resolver follows references to find the final values!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_resolution::config::ResolverConfig;

    #[test]
    fn test_registry_new() {
        let r = SettingsResolverRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsResolverRegistry::new();
        r.register("r1", SettingsResolver::new(ResolverConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_resolver_query() {
        assert!(is_resolver_query("resolve settings"));
        assert!(!is_resolver_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = resolver_fun_fact();
        assert!(fact.contains("resolve"));
    }
}
