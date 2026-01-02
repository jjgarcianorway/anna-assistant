// v0.0.699: Settings Catalog V2 (Phase 275)
// Product catalog of available settings

mod catalog;
mod helpers;
mod registry;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the same API
pub use catalog::SettingsCatalogV2;
pub use helpers::{catalog_v2_fun_fact, format_catalog_registry_v2, is_catalog_v2_query};
pub use registry::CatalogRegistryV2;
pub use types::{
    CatalogConfig, CatalogEntry, CatalogProduct, CatalogStats, CatalogStatus, CatalogType,
};
