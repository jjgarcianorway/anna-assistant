// v0.0.624: Settings Catalog (Phase 200)
// Comprehensive catalog of all available settings with documentation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Catalog section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CatalogSection {
    /// Core settings
    #[default]
    Core,
    /// User preferences
    Preferences,
    /// System settings
    System,
    /// Advanced settings
    Advanced,
    /// Experimental settings
    Experimental,
}

impl std::fmt::Display for CatalogSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Preferences => write!(f, "preferences"),
            Self::System => write!(f, "system"),
            Self::Advanced => write!(f, "advanced"),
            Self::Experimental => write!(f, "experimental"),
        }
    }
}

/// Documentation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DocLevel {
    /// Basic documentation
    #[default]
    Basic,
    /// Detailed documentation
    Detailed,
    /// Expert documentation
    Expert,
    /// Internal documentation
    Internal,
}

impl std::fmt::Display for DocLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic => write!(f, "basic"),
            Self::Detailed => write!(f, "detailed"),
            Self::Expert => write!(f, "expert"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

/// Catalog item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    /// Key
    pub key: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Section
    pub section: CatalogSection,
    /// Category
    pub category: SettingsCategory,
    /// Doc level
    pub doc_level: DocLevel,
    /// Examples
    pub examples: Vec<String>,
}

impl CatalogItem {
    /// Create new item
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        section: CatalogSection,
        category: SettingsCategory,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: description.into(),
            section,
            category,
            doc_level: DocLevel::Basic,
            examples: Vec::new(),
        }
    }

    /// Set doc level
    pub fn doc_level(mut self, level: DocLevel) -> Self {
        self.doc_level = level;
        self
    }

    /// Add example
    pub fn example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Has examples
    pub fn has_examples(&self) -> bool {
        !self.examples.is_empty()
    }

    /// Is advanced
    pub fn is_advanced(&self) -> bool {
        self.section == CatalogSection::Advanced || self.section == CatalogSection::Experimental
    }
}

/// Catalog search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching items
    pub items: Vec<CatalogItem>,
    /// Query
    pub query: String,
    /// Search time ms
    pub search_time_ms: u64,
}

impl SearchResult {
    /// Create new result
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            query: query.into(),
            search_time_ms: 0,
        }
    }

    /// Add item
    pub fn add(&mut self, item: CatalogItem) {
        self.items.push(item);
    }

    /// Set search time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.search_time_ms = ms;
        self
    }

    /// Count
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Has results
    pub fn has_results(&self) -> bool {
        !self.items.is_empty()
    }
}

/// Settings catalog
#[derive(Debug, Clone, Default)]
pub struct SettingsCatalog {
    /// Items by key
    items: HashMap<String, CatalogItem>,
    /// Items by section
    by_section: HashMap<CatalogSection, Vec<String>>,
    /// Search count
    search_count: usize,
}

impl SettingsCatalog {
    /// Create new catalog
    pub fn new() -> Self {
        Self::default()
    }

    /// Add item
    pub fn add(&mut self, item: CatalogItem) {
        let key = item.key.clone();
        let section = item.section;
        self.by_section.entry(section).or_default().push(key.clone());
        self.items.insert(key, item);
    }

    /// Remove item
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(item) = self.items.remove(key) {
            if let Some(keys) = self.by_section.get_mut(&item.section) {
                keys.retain(|k| k != key);
            }
            true
        } else {
            false
        }
    }

    /// Get item
    pub fn get(&self, key: &str) -> Option<&CatalogItem> {
        self.items.get(key)
    }

    /// List by section
    pub fn list_by_section(&self, section: CatalogSection) -> Vec<&CatalogItem> {
        self.by_section
            .get(&section)
            .map(|keys| keys.iter().filter_map(|k| self.items.get(k)).collect())
            .unwrap_or_default()
    }

    /// Search
    pub fn search(&mut self, query: &str) -> SearchResult {
        self.search_count += 1;
        let query_lower = query.to_lowercase();
        let mut result = SearchResult::new(query);
        for item in self.items.values() {
            if item.key.to_lowercase().contains(&query_lower)
                || item.title.to_lowercase().contains(&query_lower)
                || item.description.to_lowercase().contains(&query_lower)
            {
                result.add(item.clone());
            }
        }
        result
    }

    /// Item count
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Search count
    pub fn search_count(&self) -> usize {
        self.search_count
    }

    /// Section count
    pub fn section_count(&self, section: CatalogSection) -> usize {
        self.by_section.get(&section).map(|v| v.len()).unwrap_or(0)
    }
}

/// Format catalog
pub fn format_catalog(catalog: &SettingsCatalog) -> String {
    let mut output = String::new();
    output.push_str("Settings Catalog:\n");
    output.push_str(&format!("  Items: {}\n", catalog.count()));
    output.push_str(&format!("  Searches: {}\n", catalog.search_count()));
    output.push_str(&format!("  Core: {}\n", catalog.section_count(CatalogSection::Core)));
    output.push_str(&format!("  Advanced: {}\n", catalog.section_count(CatalogSection::Advanced)));
    output
}

/// Check if query is about catalog
pub fn is_catalog_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("catalog")
        || lower.contains("settings catalog")
        || lower.contains("list settings")
}

/// Fun fact about catalog
pub fn catalog_fun_fact() -> &'static str {
    "Anna's settings catalog provides comprehensive documentation for all settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_display() {
        assert_eq!(format!("{}", CatalogSection::Core), "core");
        assert_eq!(format!("{}", CatalogSection::Advanced), "advanced");
    }

    #[test]
    fn test_doc_level_display() {
        assert_eq!(format!("{}", DocLevel::Basic), "basic");
        assert_eq!(format!("{}", DocLevel::Expert), "expert");
    }

    #[test]
    fn test_item_new() {
        let i = CatalogItem::new("key", "Title", "Desc", CatalogSection::Core, SettingsCategory::Privacy);
        assert!(!i.has_examples());
    }

    #[test]
    fn test_item_example() {
        let i = CatalogItem::new("key", "Title", "Desc", CatalogSection::Core, SettingsCategory::Privacy)
            .example("example1");
        assert!(i.has_examples());
    }

    #[test]
    fn test_item_is_advanced() {
        let i = CatalogItem::new("key", "Title", "Desc", CatalogSection::Advanced, SettingsCategory::Privacy);
        assert!(i.is_advanced());
    }

    #[test]
    fn test_search_result_new() {
        let r = SearchResult::new("query");
        assert!(!r.has_results());
    }

    #[test]
    fn test_catalog_new() {
        let c = SettingsCatalog::new();
        assert_eq!(c.count(), 0);
    }

    #[test]
    fn test_catalog_add() {
        let mut c = SettingsCatalog::new();
        c.add(CatalogItem::new("k1", "T1", "D1", CatalogSection::Core, SettingsCategory::Privacy));
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn test_catalog_get() {
        let mut c = SettingsCatalog::new();
        c.add(CatalogItem::new("k1", "T1", "D1", CatalogSection::Core, SettingsCategory::Privacy));
        assert!(c.get("k1").is_some());
    }

    #[test]
    fn test_catalog_search() {
        let mut c = SettingsCatalog::new();
        c.add(CatalogItem::new("privacy", "Privacy", "Privacy settings", CatalogSection::Core, SettingsCategory::Privacy));
        let result = c.search("privacy");
        assert!(result.has_results());
    }

    #[test]
    fn test_is_catalog_query() {
        assert!(is_catalog_query("settings catalog"));
        assert!(!is_catalog_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = catalog_fun_fact();
        assert!(fact.contains("catalog"));
    }
}
