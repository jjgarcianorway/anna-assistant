// v0.0.672: Settings Projector Utilities (Phase 248)
// Helper functions for projector queries

/// Check if query is about projector
pub fn is_projector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("project") || lower.contains("select fields") || lower.contains("field projection")
}

/// Fun fact about projector
pub fn projector_fun_fact() -> &'static str {
    "Anna's settings projector creates custom views with only the fields you need!"
}
