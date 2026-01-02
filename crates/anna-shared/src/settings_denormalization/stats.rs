// v0.0.668: Denormalizer Statistics
// Statistics tracking for denormalization operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::result::DenormalizationResult;
use super::types::DenormalizationType;

/// Denormalizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DenormalizerStats {
    /// Total denormalizations
    pub total_denormalizations: usize,
    /// Keys expanded
    pub keys_expanded: usize,
    /// Keys prefixed
    pub keys_prefixed: usize,
    /// Keys suffixed
    pub keys_suffixed: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DenormalizerStats {
    /// Record denormalization
    pub fn record(&mut self, result: &DenormalizationResult) {
        self.total_denormalizations += 1;
        self.keys_expanded += result.keys_expanded;
        self.keys_prefixed += result.keys_prefixed;
        self.keys_suffixed += result.keys_suffixed;
    }

    /// Record by type
    pub fn record_type(&mut self, denorm_type: DenormalizationType) {
        *self.by_type.entry(denorm_type.to_string()).or_insert(0) += 1;
    }

    /// Changes per denormalization
    pub fn changes_per_denormalization(&self) -> f64 {
        if self.total_denormalizations == 0 {
            0.0
        } else {
            (self.keys_expanded + self.keys_prefixed + self.keys_suffixed) as f64
                / self.total_denormalizations as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = DenormalizerStats::default();
        let r = DenormalizationResult::success(HashMap::new())
            .with_counts(2, 3, 1);
        s.record(&r);
        assert_eq!(s.total_denormalizations, 1);
        assert_eq!(s.keys_prefixed, 3);
    }
}
