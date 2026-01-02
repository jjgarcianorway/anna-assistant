// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets

mod types;
mod config;
mod portfolio;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{
    PortfolioType,
    PortfolioStatus,
    PortfolioAsset,
    PortfolioHolding,
    PortfolioStats,
};

// Re-export config
pub use config::PortfolioConfig;

// Re-export portfolio
pub use portfolio::SettingsPortfolio;

// Re-export registry
pub use registry::PortfolioRegistry;

// Re-export helpers
pub use helpers::{
    format_portfolio_registry,
    is_portfolio_query,
    portfolio_fun_fact,
};
