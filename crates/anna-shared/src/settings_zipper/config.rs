// v0.0.683: Zipper Configuration
// Configuration for settings zipper operations

use serde::{Deserialize, Serialize};
use super::types::{ZipMode, UnzipMode};

/// Zipper config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipperConfig {
    /// Zip mode
    pub zip_mode: ZipMode,
    /// Unzip mode
    pub unzip_mode: UnzipMode,
    /// Default value for missing
    pub default_value: String,
    /// Pair separator
    pub pair_separator: String,
}

impl ZipperConfig {
    /// Create new config
    pub fn new(zip_mode: ZipMode) -> Self {
        Self {
            zip_mode,
            unzip_mode: UnzipMode::ByPrefix,
            default_value: "".to_string(),
            pair_separator: ":".to_string(),
        }
    }

    /// Set unzip mode
    pub fn unzip_mode(mut self, mode: UnzipMode) -> Self {
        self.unzip_mode = mode;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Set pair separator
    pub fn pair_separator(mut self, sep: impl Into<String>) -> Self {
        self.pair_separator = sep.into();
        self
    }
}

impl Default for ZipperConfig {
    fn default() -> Self {
        Self::new(ZipMode::ByKey)
    }
}
