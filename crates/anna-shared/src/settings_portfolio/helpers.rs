// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets - Helper Functions

use super::registry::PortfolioRegistry;

/// Format portfolio registry
pub fn format_portfolio_registry(registry: &PortfolioRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Portfolio Registry:\n");
    output.push_str(&format!("  Portfolios: {}\n", registry.count()));
    output
}

/// Check if query is about portfolio
pub fn is_portfolio_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings portfolio") || lower.contains("portfolio settings") || lower.contains("settings assets")
}

/// Fun fact about portfolio
pub fn portfolio_fun_fact() -> &'static str {
    "Anna's settings portfolio manages your configuration assets like investments!"
}
