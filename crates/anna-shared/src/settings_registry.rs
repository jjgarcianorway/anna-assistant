// v0.0.622: Settings Registry (Phase 198)
// Central registry for settings definitions and metadata

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Registry entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EntryType {
    /// String type
    #[default]
    String,
    /// Integer type
    Integer,
    /// Boolean type
    Boolean,
    /// Float type
    Float,
    /// Enum type
    Enum,
    /// List type
    List,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Integer => write!(f, "integer"),
            Self::Boolean => write!(f, "boolean"),
            Self::Float => write!(f, "float"),
            Self::Enum => write!(f, "enum"),
            Self::List => write!(f, "list"),
        }
    }
}

/// Registry entry visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EntryVisibility {
    /// Public - visible to all
    #[default]
    Public,
    /// Internal - system use only
    Internal,
    /// Hidden - not shown in listings
    Hidden,
    /// Deprecated - scheduled for removal
    Deprecated,
}

impl std::fmt::Display for EntryVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Internal => write!(f, "internal"),
            Self::Hidden => write!(f, "hidden"),
            Self::Deprecated => write!(f, "deprecated"),
        }
    }
}

/// Registry entry definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Key
    pub key: String,
    /// Description
    pub description: String,
    /// Category
    pub category: SettingsCategory,
    /// Entry type
    pub entry_type: EntryType,
    /// Visibility
    pub visibility: EntryVisibility,
    /// Default value
    pub default_value: Option<String>,
    /// Allowed values (for enum type)
    pub allowed_values: Vec<String>,
}

impl RegistryEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, description: impl Into<String>, category: SettingsCategory) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            category,
            entry_type: EntryType::String,
            visibility: EntryVisibility::Public,
            default_value: None,
            allowed_values: Vec::new(),
        }
    }

    /// Set entry type
    pub fn entry_type(mut self, entry_type: EntryType) -> Self {
        self.entry_type = entry_type;
        self
    }

    /// Set visibility
    pub fn visibility(mut self, visibility: EntryVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set allowed values
    pub fn allowed_values(mut self, values: Vec<String>) -> Self {
        self.allowed_values = values;
        self
    }

    /// Is public
    pub fn is_public(&self) -> bool {
        self.visibility == EntryVisibility::Public
    }

    /// Is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.visibility == EntryVisibility::Deprecated
    }
}

/// Registry lookup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupResult {
    /// Found entry
    pub entry: Option<RegistryEntry>,
    /// Lookup key
    pub key: String,
    /// Lookup time ms
    pub lookup_time_ms: u64,
}

impl LookupResult {
    /// Create found result
    pub fn found(key: impl Into<String>, entry: RegistryEntry) -> Self {
        Self {
            entry: Some(entry),
            key: key.into(),
            lookup_time_ms: 0,
        }
    }

    /// Create not found result
    pub fn not_found(key: impl Into<String>) -> Self {
        Self {
            entry: None,
            key: key.into(),
            lookup_time_ms: 0,
        }
    }

    /// Set lookup time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.lookup_time_ms = ms;
        self
    }

    /// Was found
    pub fn was_found(&self) -> bool {
        self.entry.is_some()
    }
}

/// Settings registry
#[derive(Debug, Clone, Default)]
pub struct SettingsRegistry {
    /// Entries by key
    entries: HashMap<String, RegistryEntry>,
    /// Lookup count
    lookup_count: usize,
    /// Hit count
    hit_count: usize,
}

impl SettingsRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register entry
    pub fn register(&mut self, entry: RegistryEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }

    /// Unregister entry
    pub fn unregister(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    /// Lookup entry
    pub fn lookup(&mut self, key: &str) -> LookupResult {
        self.lookup_count += 1;
        if let Some(entry) = self.entries.get(key) {
            self.hit_count += 1;
            LookupResult::found(key, entry.clone())
        } else {
            LookupResult::not_found(key)
        }
    }

    /// Get entry (without tracking)
    pub fn get(&self, key: &str) -> Option<&RegistryEntry> {
        self.entries.get(key)
    }

    /// List by category
    pub fn list_by_category(&self, category: SettingsCategory) -> Vec<&RegistryEntry> {
        self.entries.values().filter(|e| e.category == category).collect()
    }

    /// List public entries
    pub fn list_public(&self) -> Vec<&RegistryEntry> {
        self.entries.values().filter(|e| e.is_public()).collect()
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Lookup count
    pub fn lookup_count(&self) -> usize {
        self.lookup_count
    }

    /// Hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.lookup_count == 0 {
            1.0
        } else {
            self.hit_count as f64 / self.lookup_count as f64
        }
    }
}

/// Format registry
pub fn format_registry(registry: &SettingsRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Registry:\n");
    output.push_str(&format!("  Entries: {}\n", registry.count()));
    output.push_str(&format!("  Lookups: {}\n", registry.lookup_count()));
    output.push_str(&format!("  Hit Rate: {:.1}%\n", registry.hit_rate() * 100.0));
    output
}

/// Check if query is about registry
pub fn is_registry_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("registry")
        || lower.contains("settings registry")
        || lower.contains("register settings")
}

/// Fun fact about registry
pub fn registry_fun_fact() -> &'static str {
    "Anna's settings registry is the central catalog of all available settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_display() {
        assert_eq!(format!("{}", EntryType::String), "string");
        assert_eq!(format!("{}", EntryType::Boolean), "boolean");
    }

    #[test]
    fn test_visibility_display() {
        assert_eq!(format!("{}", EntryVisibility::Public), "public");
        assert_eq!(format!("{}", EntryVisibility::Hidden), "hidden");
    }

    #[test]
    fn test_entry_new() {
        let e = RegistryEntry::new("key", "desc", SettingsCategory::Privacy);
        assert!(e.is_public());
    }

    #[test]
    fn test_entry_builder() {
        let e = RegistryEntry::new("key", "desc", SettingsCategory::Risk)
            .entry_type(EntryType::Boolean)
            .visibility(EntryVisibility::Deprecated);
        assert!(e.is_deprecated());
    }

    #[test]
    fn test_lookup_found() {
        let e = RegistryEntry::new("k", "d", SettingsCategory::Privacy);
        let r = LookupResult::found("k", e);
        assert!(r.was_found());
    }

    #[test]
    fn test_lookup_not_found() {
        let r = LookupResult::not_found("missing");
        assert!(!r.was_found());
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsRegistry::new();
        r.register(RegistryEntry::new("k1", "d1", SettingsCategory::Privacy));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_registry_lookup() {
        let mut r = SettingsRegistry::new();
        r.register(RegistryEntry::new("k1", "d1", SettingsCategory::Privacy));
        let result = r.lookup("k1");
        assert!(result.was_found());
    }

    #[test]
    fn test_registry_unregister() {
        let mut r = SettingsRegistry::new();
        r.register(RegistryEntry::new("k1", "d1", SettingsCategory::Privacy));
        assert!(r.unregister("k1"));
    }

    #[test]
    fn test_is_registry_query() {
        assert!(is_registry_query("settings registry"));
        assert!(!is_registry_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = registry_fun_fact();
        assert!(fact.contains("registry"));
    }
}
