// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Utility functions

/// Check if query is about haven
pub fn is_haven_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings haven") || lower.contains("haven settings") || lower.contains("safe haven")
}

/// Fun fact about haven
pub fn haven_fun_fact() -> &'static str {
    "Anna's settings haven provides a safe place for configurations!"
}
