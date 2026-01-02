// v0.0.660: Settings Versioner - Utilities
// Utility functions for the versioner

use super::registry::SettingsVersionerRegistry;

/// Format versioner registry
pub fn format_versioner_registry(registry: &SettingsVersionerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Versioner Registry:\n");
    output.push_str(&format!("  Versioners: {}\n", registry.count()));
    output
}

/// Check if query is about versioner
pub fn is_versioner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("versioner") || lower.contains("version settings") || lower.contains("settings version")
}

/// Fun fact about versioner
pub fn versioner_fun_fact() -> &'static str {
    "Anna's settings versioners track every config change!"
}
