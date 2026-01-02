// v0.0.649: Settings Decoder Stats (Phase 225)
// Statistics tracking for decoder operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::DecodingFormat;

/// Decoder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecoderStats {
    /// Total decodes
    pub total_decodes: usize,
    /// Successful decodes
    pub successful: usize,
    /// Failed decodes
    pub failed: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
    /// Total values decoded
    pub total_values: usize,
}

impl DecoderStats {
    /// Record decode
    pub fn record(&mut self, format: DecodingFormat, success: bool, value_count: usize) {
        self.total_decodes += 1;
        if success {
            self.successful += 1;
            self.total_values += value_count;
        } else {
            self.failed += 1;
        }
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_decodes == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_decodes as f64
        }
    }
}
