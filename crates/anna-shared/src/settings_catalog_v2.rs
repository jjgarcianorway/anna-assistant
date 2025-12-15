// v0.0.699: Settings Catalog V2 (Phase 275)
// Product catalog of available settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Catalog type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CatalogType {
    /// Standard catalog
    #[default]
    Standard,
    /// Premium catalog
    Premium,
    /// Custom catalog
    Custom,
    /// Archive catalog
    Archive,
}

impl std::fmt::Display for CatalogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Premium => write!(f, "premium"),
            Self::Custom => write!(f, "custom"),
            Self::Archive => write!(f, "archive"),
        }
    }
}

/// Catalog status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CatalogStatus {
    /// Draft
    #[default]
    Draft,
    /// Published
    Published,
    /// Deprecated
    Deprecated,
    /// Archived
    Archived,
}

impl std::fmt::Display for CatalogStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Published => write!(f, "published"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

/// Catalog config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogConfig {
    /// Name
    pub name: String,
    /// Catalog type
    pub catalog_type: CatalogType,
    /// Description
    pub description: String,
    /// Max products
    pub max_products: usize,
}

impl CatalogConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            catalog_type: CatalogType::Standard,
            description: String::new(),
            max_products: 500,
        }
    }

    /// Set type
    pub fn catalog_type(mut self, ct: CatalogType) -> Self {
        self.catalog_type = ct;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set max products
    pub fn max_products(mut self, max: usize) -> Self {
        self.max_products = max;
        self
    }
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Catalog product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogProduct {
    /// Product ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Available
    pub available: bool,
}

impl CatalogProduct {
    /// Create new product
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category: String::new(),
            available: true,
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set category
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = cat.into();
        self
    }

    /// Set availability
    pub fn available(mut self, avail: bool) -> Self {
        self.available = avail;
        self
    }
}

/// Catalog entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Product ID
    pub product_id: String,
    /// Notes
    pub notes: Option<String>,
}

impl CatalogEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, product_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            product_id: product_id.into(),
            notes: None,
        }
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Catalog stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogStats {
    /// Total products
    pub total_products: usize,
    /// Available products
    pub available_products: usize,
    /// By category
    pub by_category: HashMap<String, usize>,
}

impl CatalogStats {
    /// Update from catalog
    pub fn update(&mut self, products: &[CatalogProduct]) {
        self.total_products = products.len();
        self.available_products = products.iter().filter(|p| p.available).count();
        self.by_category.clear();
        for product in products {
            if !product.category.is_empty() {
                *self.by_category.entry(product.category.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Availability rate
    pub fn availability_rate(&self) -> f64 {
        if self.total_products == 0 { 0.0 } else { self.available_products as f64 / self.total_products as f64 * 100.0 }
    }
}

/// Settings catalog v2
#[derive(Debug, Clone, Default)]
pub struct SettingsCatalogV2 {
    /// Config
    config: CatalogConfig,
    /// Products
    products: Vec<CatalogProduct>,
    /// Entries
    entries: Vec<CatalogEntry>,
    /// Status
    status: CatalogStatus,
    /// Stats
    stats: CatalogStats,
}

impl SettingsCatalogV2 {
    /// Create new catalog
    pub fn new(config: CatalogConfig) -> Self {
        Self {
            config,
            products: Vec::new(),
            entries: Vec::new(),
            status: CatalogStatus::Draft,
            stats: CatalogStats::default(),
        }
    }

    /// Add product
    pub fn add_product(&mut self, product: CatalogProduct) -> bool {
        if self.products.len() >= self.config.max_products {
            return false;
        }
        self.products.push(product);
        self.update_stats();
        true
    }

    /// Get product
    pub fn get_product(&self, id: &str) -> Option<&CatalogProduct> {
        self.products.iter().find(|p| p.id == id)
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: CatalogEntry) {
        self.entries.push(entry);
    }

    /// Get entries for product
    pub fn get_entries(&self, product_id: &str) -> Vec<&CatalogEntry> {
        self.entries.iter().filter(|e| e.product_id == product_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.products);
    }

    /// Publish
    pub fn publish(&mut self) {
        self.status = CatalogStatus::Published;
    }

    /// Deprecate
    pub fn deprecate(&mut self) {
        self.status = CatalogStatus::Deprecated;
    }

    /// Archive
    pub fn archive(&mut self) {
        self.status = CatalogStatus::Archived;
    }

    /// Get status
    pub fn status(&self) -> CatalogStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &CatalogStats {
        &self.stats
    }

    /// Product count
    pub fn product_count(&self) -> usize {
        self.products.len()
    }
}

/// Catalog registry v2
#[derive(Debug, Clone, Default)]
pub struct CatalogRegistryV2 {
    /// Catalogs by ID
    catalogs: HashMap<String, SettingsCatalogV2>,
}

impl CatalogRegistryV2 {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register catalog
    pub fn register(&mut self, id: impl Into<String>, catalog: SettingsCatalogV2) {
        self.catalogs.insert(id.into(), catalog);
    }

    /// Unregister catalog
    pub fn unregister(&mut self, id: &str) -> bool {
        self.catalogs.remove(id).is_some()
    }

    /// Get catalog
    pub fn get(&self, id: &str) -> Option<&SettingsCatalogV2> {
        self.catalogs.get(id)
    }

    /// Get catalog mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCatalogV2> {
        self.catalogs.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.catalogs.len()
    }
}

/// Format catalog registry
pub fn format_catalog_registry_v2(registry: &CatalogRegistryV2) -> String {
    let mut output = String::new();
    output.push_str("Settings Catalog V2 Registry:\n");
    output.push_str(&format!("  Catalogs: {}\n", registry.count()));
    output
}

/// Check if query is about catalog v2
pub fn is_catalog_v2_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings catalog") || lower.contains("catalog settings") || lower.contains("product catalog")
}

/// Fun fact about catalog v2
pub fn catalog_v2_fun_fact() -> &'static str {
    "Anna's settings catalog v2 organizes your configurations like a product catalog!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_type_display() {
        assert_eq!(format!("{}", CatalogType::Standard), "standard");
        assert_eq!(format!("{}", CatalogType::Premium), "premium");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CatalogStatus::Draft), "draft");
        assert_eq!(format!("{}", CatalogStatus::Published), "published");
    }

    #[test]
    fn test_config_new() {
        let c = CatalogConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CatalogConfig::new("test")
            .catalog_type(CatalogType::Premium)
            .max_products(200);
        assert_eq!(c.catalog_type, CatalogType::Premium);
        assert_eq!(c.max_products, 200);
    }

    #[test]
    fn test_product_new() {
        let p = CatalogProduct::new("p1", "Product 1");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_product_builder() {
        let p = CatalogProduct::new("p1", "Product 1")
            .category("config")
            .available(false);
        assert_eq!(p.category, "config");
        assert!(!p.available);
    }

    #[test]
    fn test_entry_new() {
        let e = CatalogEntry::new("key", "value", "p1");
        assert_eq!(e.product_id, "p1");
    }

    #[test]
    fn test_entry_notes() {
        let e = CatalogEntry::new("key", "value", "p1").notes("important");
        assert!(e.notes.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = CatalogStats::default();
        let products = vec![CatalogProduct::new("p1", "Product")];
        s.update(&products);
        assert_eq!(s.total_products, 1);
        assert_eq!(s.available_products, 1);
    }

    #[test]
    fn test_catalog_new() {
        let c = SettingsCatalogV2::new(CatalogConfig::default());
        assert_eq!(c.product_count(), 0);
    }

    #[test]
    fn test_catalog_add_product() {
        let mut c = SettingsCatalogV2::new(CatalogConfig::default());
        c.add_product(CatalogProduct::new("p1", "Product 1"));
        assert_eq!(c.product_count(), 1);
    }

    #[test]
    fn test_catalog_publish() {
        let mut c = SettingsCatalogV2::new(CatalogConfig::default());
        c.publish();
        assert_eq!(c.status(), CatalogStatus::Published);
    }

    #[test]
    fn test_catalog_deprecate() {
        let mut c = SettingsCatalogV2::new(CatalogConfig::default());
        c.deprecate();
        assert_eq!(c.status(), CatalogStatus::Deprecated);
    }

    #[test]
    fn test_registry_new() {
        let r = CatalogRegistryV2::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CatalogRegistryV2::new();
        r.register("c1", SettingsCatalogV2::new(CatalogConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_catalog_v2_query() {
        assert!(is_catalog_v2_query("settings catalog"));
        assert!(!is_catalog_v2_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = catalog_v2_fun_fact();
        assert!(fact.contains("catalog"));
    }
}
