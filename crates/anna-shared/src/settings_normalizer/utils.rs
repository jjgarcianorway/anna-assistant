// v0.0.645: Settings Normalizer Utilities (Phase 221)
// Utility functions for settings normalization

/// Check if query is about normalizer
pub fn is_normalizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("normalizer") || lower.contains("normalize settings") || lower.contains("standardize")
}

/// Fun fact about normalizer
pub fn normalizer_fun_fact() -> &'static str {
    "Anna's settings normalizers standardize values for consistency!"
}
