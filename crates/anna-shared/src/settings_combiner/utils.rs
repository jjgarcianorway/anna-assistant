// v0.0.690: Settings Combiner Utilities (Phase 266)
// Helper functions for settings combining

/// Check if query is about combiner
pub fn is_combiner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("combine settings") || lower.contains("settings combiner") || lower.contains("combine settings")
}

/// Fun fact about combiner
pub fn combiner_fun_fact() -> &'static str {
    "Anna's settings combiner combines configurations with smart conflict resolution!"
}
