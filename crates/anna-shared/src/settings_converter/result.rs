// v0.0.650: Settings Converter Result (Phase 226)
// Conversion result types

use serde::{Deserialize, Serialize};

use super::formats::{SourceFormat, TargetFormat};

/// Conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    /// Was successful
    pub success: bool,
    /// Converted data
    pub data: String,
    /// Source format
    pub source: SourceFormat,
    /// Target format
    pub target: TargetFormat,
    /// Key count
    pub key_count: usize,
}

impl ConversionResult {
    /// Create success result
    pub fn success(data: impl Into<String>, source: SourceFormat, target: TargetFormat, key_count: usize) -> Self {
        Self {
            success: true,
            data: data.into(),
            source,
            target,
            key_count,
        }
    }

    /// Create failure result
    pub fn failure(source: SourceFormat, target: TargetFormat) -> Self {
        Self {
            success: false,
            data: String::new(),
            source,
            target,
            key_count: 0,
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
