// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Portfolio type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PortfolioType {
    /// Standard portfolio
    #[default]
    Standard,
    /// Growth portfolio
    Growth,
    /// Balanced portfolio
    Balanced,
    /// Conservative portfolio
    Conservative,
}

impl std::fmt::Display for PortfolioType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Growth => write!(f, "growth"),
            Self::Balanced => write!(f, "balanced"),
            Self::Conservative => write!(f, "conservative"),
        }
    }
}

/// Portfolio status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PortfolioStatus {
    /// Active
    #[default]
    Active,
    /// Paused
    Paused,
    /// Rebalancing
    Rebalancing,
    /// Closed
    Closed,
}

impl std::fmt::Display for PortfolioStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Rebalancing => write!(f, "rebalancing"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

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

/// Portfolio asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAsset {
    /// Asset ID
    pub id: String,
    /// Name
    pub name: String,
    /// Value
    pub value: f64,
    /// Weight (percentage)
    pub weight: f64,
    /// Category
    pub category: String,
}

impl PortfolioAsset {
    /// Create new asset
    pub fn new(id: impl Into<String>, name: impl Into<String>, value: f64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            value,
            weight: 0.0,
            category: String::new(),
        }
    }

    /// Set weight
    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Set category
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = cat.into();
        self
    }
}

/// Portfolio holding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHolding {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Asset ID
    pub asset_id: String,
    /// Quantity
    pub quantity: usize,
}

impl PortfolioHolding {
    /// Create new holding
    pub fn new(key: impl Into<String>, value: impl Into<String>, asset_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            asset_id: asset_id.into(),
            quantity: 1,
        }
    }

    /// Set quantity
    pub fn quantity(mut self, qty: usize) -> Self {
        self.quantity = qty;
        self
    }
}

/// Portfolio stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortfolioStats {
    /// Total assets
    pub total_assets: usize,
    /// Total value
    pub total_value: f64,
    /// By category
    pub by_category: HashMap<String, usize>,
}

impl PortfolioStats {
    /// Update from portfolio
    pub fn update(&mut self, assets: &[PortfolioAsset]) {
        self.total_assets = assets.len();
        self.total_value = assets.iter().map(|a| a.value).sum();
        self.by_category.clear();
        for asset in assets {
            if !asset.category.is_empty() {
                *self.by_category.entry(asset.category.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Avg value per asset
    pub fn avg_value(&self) -> f64 {
        if self.total_assets == 0 { 0.0 } else { self.total_value / self.total_assets as f64 }
    }
}

/// Settings portfolio
#[derive(Debug, Clone, Default)]
pub struct SettingsPortfolio {
    /// Config
    config: PortfolioConfig,
    /// Assets
    assets: Vec<PortfolioAsset>,
    /// Holdings
    holdings: Vec<PortfolioHolding>,
    /// Status
    status: PortfolioStatus,
    /// Stats
    stats: PortfolioStats,
}

impl SettingsPortfolio {
    /// Create new portfolio
    pub fn new(config: PortfolioConfig) -> Self {
        Self {
            config,
            assets: Vec::new(),
            holdings: Vec::new(),
            status: PortfolioStatus::Active,
            stats: PortfolioStats::default(),
        }
    }

    /// Add asset
    pub fn add_asset(&mut self, asset: PortfolioAsset) -> bool {
        if self.assets.len() >= self.config.max_assets {
            return false;
        }
        self.assets.push(asset);
        self.update_stats();
        true
    }

    /// Get asset
    pub fn get_asset(&self, id: &str) -> Option<&PortfolioAsset> {
        self.assets.iter().find(|a| a.id == id)
    }

    /// Add holding
    pub fn add_holding(&mut self, holding: PortfolioHolding) {
        self.holdings.push(holding);
    }

    /// Get holdings for asset
    pub fn get_holdings(&self, asset_id: &str) -> Vec<&PortfolioHolding> {
        self.holdings.iter().filter(|h| h.asset_id == asset_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.assets);
    }

    /// Rebalance
    pub fn rebalance(&mut self) {
        self.status = PortfolioStatus::Rebalancing;
        // Recalculate weights
        let total = self.stats.total_value;
        if total > 0.0 {
            for asset in &mut self.assets {
                asset.weight = (asset.value / total) * 100.0;
            }
        }
        self.status = PortfolioStatus::Active;
    }

    /// Pause
    pub fn pause(&mut self) {
        self.status = PortfolioStatus::Paused;
    }

    /// Close
    pub fn close(&mut self) {
        self.status = PortfolioStatus::Closed;
    }

    /// Get status
    pub fn status(&self) -> PortfolioStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &PortfolioStats {
        &self.stats
    }

    /// Asset count
    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_type_display() {
        assert_eq!(format!("{}", PortfolioType::Standard), "standard");
        assert_eq!(format!("{}", PortfolioType::Growth), "growth");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PortfolioStatus::Active), "active");
        assert_eq!(format!("{}", PortfolioStatus::Rebalancing), "rebalancing");
    }

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

    #[test]
    fn test_asset_new() {
        let a = PortfolioAsset::new("a1", "Asset 1", 100.0);
        assert_eq!(a.value, 100.0);
    }

    #[test]
    fn test_asset_builder() {
        let a = PortfolioAsset::new("a1", "Asset 1", 100.0)
            .weight(25.0)
            .category("config");
        assert_eq!(a.weight, 25.0);
        assert_eq!(a.category, "config");
    }

    #[test]
    fn test_holding_new() {
        let h = PortfolioHolding::new("key", "value", "a1");
        assert_eq!(h.asset_id, "a1");
    }

    #[test]
    fn test_holding_quantity() {
        let h = PortfolioHolding::new("key", "value", "a1").quantity(5);
        assert_eq!(h.quantity, 5);
    }

    #[test]
    fn test_stats_update() {
        let mut s = PortfolioStats::default();
        let assets = vec![PortfolioAsset::new("a1", "Asset", 100.0)];
        s.update(&assets);
        assert_eq!(s.total_assets, 1);
        assert_eq!(s.total_value, 100.0);
    }

    #[test]
    fn test_portfolio_new() {
        let p = SettingsPortfolio::new(PortfolioConfig::default());
        assert_eq!(p.asset_count(), 0);
    }

    #[test]
    fn test_portfolio_add_asset() {
        let mut p = SettingsPortfolio::new(PortfolioConfig::default());
        p.add_asset(PortfolioAsset::new("a1", "Asset 1", 100.0));
        assert_eq!(p.asset_count(), 1);
    }

    #[test]
    fn test_portfolio_rebalance() {
        let mut p = SettingsPortfolio::new(PortfolioConfig::default());
        p.add_asset(PortfolioAsset::new("a1", "Asset 1", 100.0));
        p.rebalance();
        assert_eq!(p.status(), PortfolioStatus::Active);
    }

    #[test]
    fn test_portfolio_pause() {
        let mut p = SettingsPortfolio::new(PortfolioConfig::default());
        p.pause();
        assert_eq!(p.status(), PortfolioStatus::Paused);
    }

    #[test]
    fn test_registry_new() {
        let r = PortfolioRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PortfolioRegistry::new();
        r.register("p1", SettingsPortfolio::new(PortfolioConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_portfolio_query() {
        assert!(is_portfolio_query("settings portfolio"));
        assert!(!is_portfolio_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = portfolio_fun_fact();
        assert!(fact.contains("portfolio"));
    }
}
