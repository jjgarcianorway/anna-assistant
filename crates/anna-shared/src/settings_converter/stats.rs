// v0.0.650: Settings Converter Stats (Phase 226)
// Statistics for conversion operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::formats::{SourceFormat, TargetFormat};

/// Converter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConverterStats {
    /// Total conversions
    pub total_conversions: usize,
    /// Successful conversions
    pub successful: usize,
    /// Failed conversions
    pub failed: usize,
    /// By source
    pub by_source: HashMap<String, usize>,
    /// By target
    pub by_target: HashMap<String, usize>,
}

impl ConverterStats {
    /// Record conversion
    pub fn record(&mut self, source: SourceFormat, target: TargetFormat, success: bool) {
        self.total_conversions += 1;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        *self.by_source.entry(source.to_string()).or_insert(0) += 1;
        *self.by_target.entry(target.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_conversions == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_conversions as f64
        }
    }
}
