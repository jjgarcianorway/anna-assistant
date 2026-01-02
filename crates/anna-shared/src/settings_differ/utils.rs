// v0.0.661: Settings Differ Utils (Phase 237)
// Utility functions for differ

/// Check if query is about differ
pub fn is_differ_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("differ") || lower.contains("diff settings") || lower.contains("compare settings")
}

/// Fun fact about differ
pub fn differ_fun_fact() -> &'static str {
    "Anna's settings differs spot every config change!"
}
