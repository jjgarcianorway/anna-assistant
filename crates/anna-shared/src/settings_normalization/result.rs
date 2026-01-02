// v0.0.667: Settings Normalization (Phase 243)
// Result and stats types for normalization operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    /// Normalized settings
    pub settings: HashMap<String, String>,
    /// Keys normalized
    pub keys_normalized: usize,
    /// Values normalized
    pub values_normalized: usize,
    /// Keys removed
    pub keys_removed: usize,
    /// Success
    pub success: bool,
}

impl NormalizationResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            keys_normalized: 0,
            values_normalized: 0,
            keys_removed: 0,
            success: true,
        }
    }

    /// With counts
    pub fn with_counts(mut self, keys: usize, values: usize, removed: usize) -> Self {
        self.keys_normalized = keys;
        self.values_normalized = values;
        self.keys_removed = removed;
        self
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.keys_normalized + self.values_normalized + self.keys_removed
    }
}

impl Default for NormalizationResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

/// Normalizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizerStats {
    /// Total normalizations
    pub total_normalizations: usize,
    /// Keys normalized
    pub keys_normalized: usize,
    /// Values normalized
    pub values_normalized: usize,
    /// Keys removed
    pub keys_removed: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NormalizerStats {
    /// Record normalization
    pub fn record(&mut self, result: &NormalizationResult) {
        self.total_normalizations += 1;
        self.keys_normalized += result.keys_normalized;
        self.values_normalized += result.values_normalized;
        self.keys_removed += result.keys_removed;
    }

    /// Record by type
    pub fn record_type(&mut self, norm_type: crate::settings_normalization::NormalizationType) {
        *self.by_type.entry(norm_type.to_string()).or_insert(0) += 1;
    }

    /// Changes per normalization
    pub fn changes_per_normalization(&self) -> f64 {
        if self.total_normalizations == 0 {
            0.0
        } else {
            (self.keys_normalized + self.values_normalized) as f64 / self.total_normalizations as f64
        }
    }
}
