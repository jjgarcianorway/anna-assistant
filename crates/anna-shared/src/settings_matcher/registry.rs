// v0.0.687: Settings Matcher Registry (Phase 263)
// Registry for managing multiple matchers

use std::collections::HashMap;

use super::matcher::SettingsMatcher;

/// Matcher registry
#[derive(Debug, Clone, Default)]
pub struct MatcherRegistry {
    /// Matchers by ID
    matchers: HashMap<String, SettingsMatcher>,
}

impl MatcherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register matcher
    pub fn register(&mut self, id: impl Into<String>, matcher: SettingsMatcher) {
        self.matchers.insert(id.into(), matcher);
    }

    /// Unregister matcher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.matchers.remove(id).is_some()
    }

    /// Get matcher
    pub fn get(&self, id: &str) -> Option<&SettingsMatcher> {
        self.matchers.get(id)
    }

    /// Get matcher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMatcher> {
        self.matchers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.matchers.len()
    }
}

/// Format matcher registry
pub fn format_matcher_registry(registry: &MatcherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Matcher Registry:\n");
    output.push_str(&format!("  Matchers: {}\n", registry.count()));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_matcher::types::MatcherConfig;

    #[test]
    fn test_registry_new() {
        let r = MatcherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MatcherRegistry::new();
        r.register("m1", SettingsMatcher::new(MatcherConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
