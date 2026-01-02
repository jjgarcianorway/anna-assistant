// v0.0.648: Settings Encoder (Phase 224)
// Encoder statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::format::{EncodingFormat, EncodingOptions};

/// Encoder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncoderStats {
    /// Total encodes
    pub total_encodes: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
    /// By options
    pub by_options: HashMap<String, usize>,
    /// Total bytes encoded
    pub total_bytes: usize,
}

impl EncoderStats {
    /// Record encode
    pub fn record(&mut self, format: EncodingFormat, options: EncodingOptions, byte_size: usize) {
        self.total_encodes += 1;
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
        *self.by_options.entry(options.to_string()).or_insert(0) += 1;
        self.total_bytes += byte_size;
    }

    /// Average bytes per encode
    pub fn average_bytes(&self) -> f64 {
        if self.total_encodes == 0 {
            0.0
        } else {
            self.total_bytes as f64 / self.total_encodes as f64
        }
    }
}
