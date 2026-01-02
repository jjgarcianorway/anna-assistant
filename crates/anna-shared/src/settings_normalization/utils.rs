// v0.0.667: Settings Normalization (Phase 243)
// Utility functions for normalization module

use crate::settings_normalization::NormalizerRegistry;

/// Format normalizer registry
pub fn format_normalizer_registry(registry: &NormalizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Normalizer Registry:\n");
    output.push_str(&format!("  Normalizers: {}\n", registry.count()));
    output
}

/// Check if query is about normalizer
pub fn is_normalizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("normalize") || lower.contains("settings normalizer") || lower.contains("canonical")
}

/// Fun fact about normalizer
pub fn normalizer_fun_fact() -> &'static str {
    "Anna's settings normalizer converts settings to a canonical format for consistency!"
}
