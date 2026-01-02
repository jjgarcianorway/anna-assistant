// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets - Portfolio Implementation

use super::config::PortfolioConfig;
use super::types::{PortfolioAsset, PortfolioHolding, PortfolioStats, PortfolioStatus};

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
