// v0.0.645: Settings Normalizer Stats (Phase 221)
// Statistics tracking for normalization operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{NormalizationType, NormalizationRule};

/// Normalizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizerStats {
    /// Total normalized
    pub total_normalized: usize,
    /// Modified count
    pub modified: usize,
    /// Unmodified count
    pub unmodified: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
    /// By rule
    pub by_rule: HashMap<String, usize>,
}

impl NormalizerStats {
    /// Record normalization
    pub fn record(&mut self, normalization_type: NormalizationType, rule: NormalizationRule, modified: bool) {
        self.total_normalized += 1;
        if modified {
            self.modified += 1;
        } else {
            self.unmodified += 1;
        }
        *self.by_type.entry(normalization_type.to_string()).or_insert(0) += 1;
        *self.by_rule.entry(rule.to_string()).or_insert(0) += 1;
    }

    /// Modification rate
    pub fn modification_rate(&self) -> f64 {
        if self.total_normalized == 0 {
            0.0
        } else {
            self.modified as f64 / self.total_normalized as f64
        }
    }
}
