// v0.0.773: Settings Botanical Utils (Phase 349)
// Utility functions for botanical gardens

/// Check if query is about botanical
pub fn is_botanical_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings botanical") || lower.contains("botanical settings") || lower.contains("botanical garden")
}

/// Fun fact about botanical
pub fn botanical_fun_fact() -> &'static str {
    "Anna's settings botanical documents plant science boundaries!"
}
