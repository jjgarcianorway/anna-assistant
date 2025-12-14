// v0.0.606: Settings Bundler (Phase 182)
// Bundle settings into portable packages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Bundle format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BundleFormat {
    /// JSON bundle
    #[default]
    Json,
    /// Binary bundle
    Binary,
    /// Compressed bundle
    Compressed,
    /// Encrypted bundle
    Encrypted,
}

impl std::fmt::Display for BundleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Binary => write!(f, "binary"),
            Self::Compressed => write!(f, "compressed"),
            Self::Encrypted => write!(f, "encrypted"),
        }
    }
}

/// Bundle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BundleStatus {
    /// Created
    Created,
    /// Valid
    Valid,
    /// Invalid
    Invalid,
    /// Expired
    Expired,
}

impl std::fmt::Display for BundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Valid => write!(f, "valid"),
            Self::Invalid => write!(f, "invalid"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

/// Bundle metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMeta {
    /// Bundle ID
    pub id: String,
    /// Name
    pub name: String,
    /// Version
    pub version: String,
    /// Format
    pub format: BundleFormat,
    /// Created timestamp
    pub created: u64,
    /// Description
    pub description: String,
}

impl BundleMeta {
    /// Create new metadata
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: "1.0".to_string(),
            format: BundleFormat::Json,
            created: 0,
            description: String::new(),
        }
    }

    /// Set version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set format
    pub fn format(mut self, format: BundleFormat) -> Self {
        self.format = format;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Bundle entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    /// Key
    pub key: String,
    /// Category
    pub category: SettingsCategory,
    /// Value
    pub value: String,
}

impl BundleEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, category: SettingsCategory, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            category,
            value: value.into(),
        }
    }
}

/// Settings bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsBundle {
    /// Metadata
    pub meta: BundleMeta,
    /// Entries
    pub entries: Vec<BundleEntry>,
    /// Status
    pub status: BundleStatus,
}

impl SettingsBundle {
    /// Create new bundle
    pub fn new(meta: BundleMeta) -> Self {
        Self {
            meta,
            entries: Vec::new(),
            status: BundleStatus::Created,
        }
    }

    /// Add entry
    pub fn add(&mut self, entry: BundleEntry) {
        self.entries.push(entry);
    }

    /// Get entries by category
    pub fn by_category(&self, category: SettingsCategory) -> Vec<&BundleEntry> {
        self.entries.iter().filter(|e| e.category == category).collect()
    }

    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Category count
    pub fn category_count(&self) -> usize {
        let cats: std::collections::HashSet<_> = self.entries.iter().map(|e| e.category).collect();
        cats.len()
    }

    /// Mark valid
    pub fn mark_valid(&mut self) {
        self.status = BundleStatus::Valid;
    }

    /// Mark invalid
    pub fn mark_invalid(&mut self) {
        self.status = BundleStatus::Invalid;
    }

    /// Is valid
    pub fn is_valid(&self) -> bool {
        self.status == BundleStatus::Valid
    }
}

/// Bundle manager
#[derive(Debug, Clone, Default)]
pub struct BundleManager {
    /// Bundles by ID
    bundles: HashMap<String, SettingsBundle>,
}

impl BundleManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add bundle
    pub fn add(&mut self, bundle: SettingsBundle) {
        self.bundles.insert(bundle.meta.id.clone(), bundle);
    }

    /// Remove bundle
    pub fn remove(&mut self, id: &str) -> Option<SettingsBundle> {
        self.bundles.remove(id)
    }

    /// Get bundle
    pub fn get(&self, id: &str) -> Option<&SettingsBundle> {
        self.bundles.get(id)
    }

    /// Get bundle mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBundle> {
        self.bundles.get_mut(id)
    }

    /// List IDs
    pub fn list_ids(&self) -> Vec<&String> {
        self.bundles.keys().collect()
    }

    /// Bundle count
    pub fn count(&self) -> usize {
        self.bundles.len()
    }

    /// Valid count
    pub fn valid_count(&self) -> usize {
        self.bundles.values().filter(|b| b.is_valid()).count()
    }
}

/// Format bundle
pub fn format_bundle(bundle: &SettingsBundle) -> String {
    let mut output = String::new();
    output.push_str(&format!("Bundle: {} ({})\n", bundle.meta.name, bundle.meta.id));
    output.push_str(&format!("  Version: {}\n", bundle.meta.version));
    output.push_str(&format!("  Format: {}\n", bundle.meta.format));
    output.push_str(&format!("  Status: {}\n", bundle.status));
    output.push_str(&format!("  Entries: {}\n", bundle.entry_count()));
    output.push_str(&format!("  Categories: {}\n", bundle.category_count()));
    output
}

/// Check if query is about bundler
pub fn is_bundler_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("bundle")
        || lower.contains("package settings")
        || lower.contains("export bundle")
}

/// Fun fact about bundler
pub fn bundler_fun_fact() -> &'static str {
    "Anna can bundle your settings into portable packages for backup or sharing!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", BundleFormat::Json), "json");
        assert_eq!(format!("{}", BundleFormat::Compressed), "compressed");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BundleStatus::Valid), "valid");
        assert_eq!(format!("{}", BundleStatus::Invalid), "invalid");
    }

    #[test]
    fn test_meta_new() {
        let m = BundleMeta::new("b1", "Test Bundle");
        assert_eq!(m.id, "b1");
    }

    #[test]
    fn test_meta_builder() {
        let m = BundleMeta::new("b1", "Test")
            .version("2.0")
            .format(BundleFormat::Compressed);
        assert_eq!(m.version, "2.0");
    }

    #[test]
    fn test_entry_new() {
        let e = BundleEntry::new("key", SettingsCategory::Personality, "value");
        assert_eq!(e.key, "key");
    }

    #[test]
    fn test_bundle_new() {
        let b = SettingsBundle::new(BundleMeta::new("b1", "Test"));
        assert_eq!(b.entry_count(), 0);
    }

    #[test]
    fn test_bundle_add() {
        let mut b = SettingsBundle::new(BundleMeta::new("b1", "T"));
        b.add(BundleEntry::new("k", SettingsCategory::Privacy, "v"));
        assert_eq!(b.entry_count(), 1);
    }

    #[test]
    fn test_bundle_valid() {
        let mut b = SettingsBundle::new(BundleMeta::new("b1", "T"));
        b.mark_valid();
        assert!(b.is_valid());
    }

    #[test]
    fn test_manager_new() {
        let m = BundleManager::new();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_manager_add_remove() {
        let mut m = BundleManager::new();
        m.add(SettingsBundle::new(BundleMeta::new("b1", "T")));
        assert_eq!(m.count(), 1);
        m.remove("b1");
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_is_bundler_query() {
        assert!(is_bundler_query("bundle settings"));
        assert!(!is_bundler_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = bundler_fun_fact();
        assert!(fact.contains("bundle"));
    }
}
