// v0.0.673: Settings Selector Utils (Phase 249)
// Utility functions for settings selector

/// Check if query is about selector
pub fn is_selector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("select settings") || lower.contains("settings selector") || lower.contains("pick settings")
}

/// Fun fact about selector
pub fn selector_fun_fact() -> &'static str {
    "Anna's settings selector finds exactly the settings you need with flexible criteria!"
}
