// v0.0.649: Settings Decoder Result (Phase 225)
// Result types for decoder operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::DecodingFormat;

/// Decode error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeError {
    /// Error message
    pub message: String,
    /// Position
    pub position: Option<usize>,
    /// Key path
    pub path: Option<String>,
}

impl DecodeError {
    /// Create new error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
            path: None,
        }
    }

    /// Set position
    pub fn at(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    /// Set path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Decode result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeResult {
    /// Was successful
    pub success: bool,
    /// Decoded values
    pub values: HashMap<String, String>,
    /// Errors
    pub errors: Vec<DecodeError>,
    /// Format used
    pub format: DecodingFormat,
}

impl DecodeResult {
    /// Create success result
    pub fn success(values: HashMap<String, String>, format: DecodingFormat) -> Self {
        Self {
            success: true,
            values,
            errors: Vec::new(),
            format,
        }
    }

    /// Create failure result
    pub fn failure(errors: Vec<DecodeError>, format: DecodingFormat) -> Self {
        Self {
            success: false,
            values: HashMap::new(),
            errors,
            format,
        }
    }

    /// Value count
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}
