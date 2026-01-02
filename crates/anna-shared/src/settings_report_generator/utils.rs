// v0.0.640: Settings Report Generator - Utils (Phase 216)
// Utility functions for reporter

use super::registry::SettingsReporterRegistry;

/// Format reporter registry
pub fn format_reporter_registry(registry: &SettingsReporterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Reporter Registry:\n");
    output.push_str(&format!("  Reporters: {}\n", registry.count()));
    output
}

/// Check if query is about reporter
pub fn is_reporter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("reporter") || lower.contains("report settings") || lower.contains("generate report")
}

/// Fun fact about reporter
pub fn reporter_fun_fact() -> &'static str {
    "Anna's settings reporters generate multi-format status reports!"
}
