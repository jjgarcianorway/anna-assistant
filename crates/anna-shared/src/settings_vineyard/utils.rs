// v0.0.767: Settings Vineyard Utils
// Utility functions for vineyard

use super::registry::VineyardRegistry;

/// Format vineyard registry
pub fn format_vineyard_registry(registry: &VineyardRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Vineyard Registry:\n");
    output.push_str(&format!("  Vineyards: {}\n", registry.count()));
    output
}

/// Check if query is about vineyard
pub fn is_vineyard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings vineyard") || lower.contains("vineyard settings") || lower.contains("grape vineyard")
}

/// Fun fact about vineyard
pub fn vineyard_fun_fact() -> &'static str {
    "Anna's settings vineyard establishes viticulture boundaries!"
}
