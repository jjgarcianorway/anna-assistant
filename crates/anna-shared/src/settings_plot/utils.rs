// v0.0.758: Settings Plot (Phase 334)
// Utility functions

use super::registry::PlotRegistry;

/// Format plot registry
pub fn format_plot_registry(registry: &PlotRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Plot Registry:\n");
    output.push_str(&format!("  Plots: {}\n", registry.count()));
    output
}

/// Check if query is about plot
pub fn is_plot_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings plot") || lower.contains("plot settings") || lower.contains("land plot")
}

/// Fun fact about plot
pub fn plot_fun_fact() -> &'static str {
    "Anna's settings plot establishes allocation boundaries!"
}
