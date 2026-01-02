// v0.0.599: Settings Resolver Configuration (Phase 175)
// Configuration for conflict resolution strategies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;
use super::types::ResolutionStrategy;

/// Resolver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverConfig {
    /// Default strategy
    pub default_strategy: ResolutionStrategy,
    /// Per-category strategies
    pub category_strategies: HashMap<SettingsCategory, ResolutionStrategy>,
    /// Auto-resolve
    pub auto_resolve: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            default_strategy: ResolutionStrategy::Last,
            category_strategies: HashMap::new(),
            auto_resolve: true,
        }
    }
}

impl ResolverConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default strategy
    pub fn default_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Set category strategy
    pub fn category_strategy(mut self, category: SettingsCategory, strategy: ResolutionStrategy) -> Self {
        self.category_strategies.insert(category, strategy);
        self
    }

    /// Get strategy for category
    pub fn strategy_for(&self, category: SettingsCategory) -> ResolutionStrategy {
        self.category_strategies
            .get(&category)
            .copied()
            .unwrap_or(self.default_strategy)
    }
}
