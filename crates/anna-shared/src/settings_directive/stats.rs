// v0.0.718: Settings Directive Stats (Phase 294)
// Statistics for directive systems

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::DirectiveType;
use super::order::DirectiveOrder;

/// Directive stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DirectiveStats {
    /// Total directives
    pub total_directives: usize,
    /// Enforced directives
    pub enforced: usize,
    /// Mandatory count
    pub mandatory_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DirectiveStats {
    /// Update from orders
    pub fn update(&mut self, orders: &[DirectiveOrder], directive_type: DirectiveType) {
        self.total_directives = orders.len();
        self.enforced = orders.iter().filter(|o| o.enforced).count();
        if directive_type == DirectiveType::Mandatory {
            self.mandatory_count = orders.len();
        }
        *self.by_type.entry(directive_type.to_string()).or_insert(0) += 1;
    }

    /// Enforcement rate
    pub fn enforcement_rate(&self) -> f64 {
        if self.total_directives == 0 { 0.0 } else { self.enforced as f64 / self.total_directives as f64 * 100.0 }
    }
}
