// v0.0.699: Settings Catalog V2 (Phase 275) - Helpers
// Helper and utility functions

use super::registry::CatalogRegistryV2;

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
