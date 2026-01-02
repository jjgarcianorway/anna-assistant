// v0.0.748: Settings District Utils (Phase 324)
// Utility functions for district operations

use super::registry::DistrictRegistry;

/// Format district registry
pub fn format_district_registry(registry: &DistrictRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings District Registry:\n");
    output.push_str(&format!("  Districts: {}\n", registry.count()));
    output
}

/// Check if query is about district
pub fn is_district_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings district") || lower.contains("district settings") || lower.contains("local district")
}

/// Fun fact about district
pub fn district_fun_fact() -> &'static str {
    "Anna's settings district establishes local administration!"
}
