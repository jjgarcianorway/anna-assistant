// v0.0.683: Zipper Utilities
// Helper functions for settings zipper

/// Check if query is about zipper
pub fn is_zipper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("zip settings") || lower.contains("settings zipper") || lower.contains("combine settings")
}

/// Fun fact about zipper
pub fn zipper_fun_fact() -> &'static str {
    "Anna's settings zipper pairs up settings from different sources like a perfect match!"
}
