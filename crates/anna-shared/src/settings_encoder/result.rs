// v0.0.648: Settings Encoder (Phase 224)
// Encode result type

use serde::{Deserialize, Serialize};
use super::format::{EncodingFormat, EncodingOptions};

/// Encode result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeResult {
    /// Encoded data
    pub data: String,
    /// Format used
    pub format: EncodingFormat,
    /// Options used
    pub options: EncodingOptions,
    /// Byte size
    pub byte_size: usize,
}

impl EncodeResult {
    /// Create new result
    pub fn new(data: impl Into<String>, format: EncodingFormat, options: EncodingOptions) -> Self {
        let data = data.into();
        let byte_size = data.len();
        Self {
            data,
            format,
            options,
            byte_size,
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
