// v0.0.650: Settings Converter Utils (Phase 226)
// Utility functions for settings conversion

/// Check if query is about converter
pub fn is_converter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("converter") || lower.contains("convert settings") || lower.contains("transform format")
}

/// Fun fact about converter
pub fn converter_fun_fact() -> &'static str {
    "Anna's settings converters transform configs between any format!"
}
