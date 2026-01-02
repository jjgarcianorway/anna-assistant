// v0.0.718: Settings Directive Core (Phase 294)
// Main directive system implementation

use super::config::DirectiveConfig;
use super::order::{DirectiveOrder, DirectiveSupplement};
use super::stats::DirectiveStats;

/// Settings directive
#[derive(Debug, Clone, Default)]
pub struct SettingsDirective {
    /// Config
    config: DirectiveConfig,
    /// Orders
    orders: Vec<DirectiveOrder>,
    /// Supplements
    supplements: Vec<DirectiveSupplement>,
    /// Stats
    stats: DirectiveStats,
}

impl SettingsDirective {
    /// Create new directive system
    pub fn new(config: DirectiveConfig) -> Self {
        Self {
            config,
            orders: Vec::new(),
            supplements: Vec::new(),
            stats: DirectiveStats::default(),
        }
    }

    /// Add order
    pub fn add_order(&mut self, order: DirectiveOrder) -> bool {
        if self.orders.len() >= self.config.max_directives {
            return false;
        }
        self.orders.push(order);
        self.update_stats();
        true
    }

    /// Get order
    pub fn get_order(&self, id: &str) -> Option<&DirectiveOrder> {
        self.orders.iter().find(|o| o.id == id)
    }

    /// Get order mut
    pub fn get_order_mut(&mut self, id: &str) -> Option<&mut DirectiveOrder> {
        self.orders.iter_mut().find(|o| o.id == id)
    }

    /// Add supplement
    pub fn add_supplement(&mut self, supplement: DirectiveSupplement) {
        self.supplements.push(supplement);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.orders, self.config.directive_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DirectiveStats {
        &self.stats
    }

    /// Order count
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }
}
