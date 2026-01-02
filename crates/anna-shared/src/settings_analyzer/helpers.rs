// v0.0.642: Settings Analyzer (Phase 218)
// Helper functions for settings analyzer

use crate::settings_analyzer::registry::SettingsAnalyzerRegistry;

/// Format analyzer registry
pub fn format_analyzer_registry(registry: &SettingsAnalyzerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Analyzer Registry:\n");
    output.push_str(&format!("  Analyzers: {}\n", registry.count()));
    output
}

/// Check if query is about analyzer
pub fn is_analyzer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("analyzer") || lower.contains("analyze settings") || lower.contains("pattern")
}

/// Fun fact about analyzer
pub fn analyzer_fun_fact() -> &'static str {
    "Anna's settings analyzers detect patterns and anomalies!"
}
