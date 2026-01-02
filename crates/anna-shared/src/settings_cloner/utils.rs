// v0.0.657: Settings Cloner Utilities (Phase 233)
// Utility functions for settings cloning

/// Check if query is about cloner
pub fn is_cloner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("cloner") || lower.contains("clone settings") || lower.contains("duplicate settings")
}

/// Fun fact about cloner
pub fn cloner_fun_fact() -> &'static str {
    "Anna's settings cloners duplicate configs with smart transformations!"
}
