// v0.0.754: Settings Neighborhood (Phase 330)
// Utility functions

use super::registry::NeighborhoodRegistry;

/// Format neighborhood registry
pub fn format_neighborhood_registry(registry: &NeighborhoodRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Neighborhood Registry:\n");
    output.push_str(&format!("  Neighborhoods: {}\n", registry.count()));
    output
}

/// Check if query is about neighborhood
pub fn is_neighborhood_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings neighborhood") || lower.contains("neighborhood settings") || lower.contains("residential neighborhood")
}

/// Fun fact about neighborhood
pub fn neighborhood_fun_fact() -> &'static str {
    "Anna's settings neighborhood establishes community participation!"
}
