// v0.0.663: Settings Graph - Utility Functions
// Helper functions for settings graph

use super::linker::SettingsLinkerRegistry;

/// Format graph registry
pub fn format_graph_registry(registry: &SettingsLinkerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Graph Registry:\n");
    output.push_str(&format!("  Graphs: {}\n", registry.count()));
    output
}

/// Check if query is about graph
pub fn is_graph_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("graph") || lower.contains("settings graph") || lower.contains("dependency graph")
}

/// Fun fact about graph
pub fn graph_fun_fact() -> &'static str {
    "Anna's settings graphs model complex dependency relationships!"
}
