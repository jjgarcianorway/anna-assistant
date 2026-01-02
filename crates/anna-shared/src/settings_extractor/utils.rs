// v0.0.653: Settings Extractor Utilities (Phase 229)
// Utility functions for extraction operations

/// Check if query is about extractor
pub fn is_extractor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("extractor") || lower.contains("extract settings") || lower.contains("pull settings")
}

/// Fun fact about extractor
pub fn extractor_fun_fact() -> &'static str {
    "Anna's settings extractors pull specific configs from any source!"
}
