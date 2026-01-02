// v0.0.698: Settings Portfolio - Config (Phase 274)
// Portfolio configuration

use serde::{Deserialize, Serialize};
use super::types::PortfolioType;

/// Portfolio config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioConfig {
    /// Name
    pub name: String,
    /// Portfolio type
    pub portfolio_type: PortfolioType,
    /// Description
    pub description: String,
    /// Max assets
    pub max_assets: usize,
}

impl PortfolioConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            portfolio_type: PortfolioType::Standard,
            description: String::new(),
            max_assets: 100,
        }
    }

    /// Set type
    pub fn portfolio_type(mut self, pt: PortfolioType) -> Self {
        self.portfolio_type = pt;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set max assets
    pub fn max_assets(mut self, max: usize) -> Self {
        self.max_assets = max;
        self
    }
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = PortfolioConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PortfolioConfig::new("test")
            .portfolio_type(PortfolioType::Balanced)
            .max_assets(50);
        assert_eq!(c.portfolio_type, PortfolioType::Balanced);
        assert_eq!(c.max_assets, 50);
    }
}
