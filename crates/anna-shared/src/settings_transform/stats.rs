// v0.0.666: Settings Transform Stats (Phase 242)
// Statistics tracking for transformations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::TransformType;
use super::rule::TransformResult;

/// Transformer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformerStats {
    /// Total transformations
    pub total_transformations: usize,
    /// Keys transformed
    pub keys_transformed: usize,
    /// Rules applied
    pub rules_applied: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TransformerStats {
    /// Record transformation
    pub fn record(&mut self, result: &TransformResult) {
        self.total_transformations += 1;
        self.keys_transformed += result.keys_transformed;
        self.rules_applied += result.rules_applied.len();
    }

    /// Record by type
    pub fn record_type(&mut self, transform_type: TransformType) {
        *self.by_type.entry(transform_type.to_string()).or_insert(0) += 1;
    }

    /// Keys per transformation
    pub fn keys_per_transformation(&self) -> f64 {
        if self.total_transformations == 0 {
            0.0
        } else {
            self.keys_transformed as f64 / self.total_transformations as f64
        }
    }

    /// Rules per transformation
    pub fn rules_per_transformation(&self) -> f64 {
        if self.total_transformations == 0 {
            0.0
        } else {
            self.rules_applied as f64 / self.total_transformations as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = TransformerStats::default();
        let r = TransformResult::success(HashMap::new()).with_transformed(5);
        s.record(&r);
        assert_eq!(s.total_transformations, 1);
        assert_eq!(s.keys_transformed, 5);
    }
}
