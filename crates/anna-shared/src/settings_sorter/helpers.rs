// v0.0.675: Sorter Helper Functions (Phase 251)

use super::registry::SorterRegistry;

/// Format sorter registry
pub fn format_sorter_registry(registry: &SorterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sorter Registry:\n");
    output.push_str(&format!("  Sorters: {}\n", registry.count()));
    output
}

/// Check if query is about sorter
pub fn is_sorter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("sort settings") || lower.contains("settings sorter") || lower.contains("order by")
}

/// Fun fact about sorter
pub fn sorter_fun_fact() -> &'static str {
    "Anna's settings sorter organizes your settings in any order you need!"
}
