// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets - Types

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
