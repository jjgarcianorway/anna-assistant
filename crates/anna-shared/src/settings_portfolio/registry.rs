// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets - Registry

use super::portfolio::SettingsPortfolio;
use std::collections::HashMap;

/// Portfolio registry
#[derive(Debug, Clone, Default)]
pub struct PortfolioRegistry {
    /// Portfolios by ID
    portfolios: HashMap<String, SettingsPortfolio>,
}

impl PortfolioRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register portfolio
    pub fn register(&mut self, id: impl Into<String>, portfolio: SettingsPortfolio) {
        self.portfolios.insert(id.into(), portfolio);
    }

    /// Unregister portfolio
    pub fn unregister(&mut self, id: &str) -> bool {
        self.portfolios.remove(id).is_some()
    }

    /// Get portfolio
    pub fn get(&self, id: &str) -> Option<&SettingsPortfolio> {
        self.portfolios.get(id)
    }

    /// Get portfolio mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPortfolio> {
        self.portfolios.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.portfolios.len()
    }
}
