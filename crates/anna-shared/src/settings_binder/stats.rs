// v0.0.652: Settings Binder - Stats
// Statistics tracking for bindings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::BindingType;

/// Binder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinderStats {
    /// Total bind attempts
    pub total_binds: usize,
    /// Successful binds
    pub successful: usize,
    /// Failed binds
    pub failed: usize,
    /// By binding type
    pub by_type: HashMap<String, usize>,
}

impl BinderStats {
    /// Record bind
    pub fn record(&mut self, binding_type: BindingType, success: bool) {
        self.total_binds += 1;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        *self.by_type.entry(binding_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_binds == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_binds as f64
        }
    }
}
