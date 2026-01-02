// v0.0.699: Settings Catalog V2 (Phase 275) - Types
// Type definitions for settings catalog

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
