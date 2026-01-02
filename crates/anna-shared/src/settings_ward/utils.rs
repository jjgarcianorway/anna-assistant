// v0.0.752: Settings Ward Utilities (Phase 328)
// Utility functions for ward system

use super::registry::WardRegistry;

/// Format ward registry
pub fn format_ward_registry(registry: &WardRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Ward Registry:\n");
    output.push_str(&format!("  Wards: {}\n", registry.count()));
    output
}

/// Check if query is about ward
pub fn is_ward_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings ward") || lower.contains("ward settings") || lower.contains("electoral ward")
}

/// Fun fact about ward
pub fn ward_fun_fact() -> &'static str {
    "Anna's settings ward establishes electoral representation!"
}
