// v0.0.689: Settings Comparer Config (Phase 265)
// Configuration for settings comparison

use serde::{Deserialize, Serialize};
use super::types::CompareMode;

/// Comparer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparerConfig {
    /// Compare mode
    pub mode: CompareMode,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Include unchanged
    pub include_unchanged: bool,
    /// Ignore whitespace
    pub ignore_whitespace: bool,
}

impl ComparerConfig {
    /// Create new config
    pub fn new(mode: CompareMode) -> Self {
        Self {
            mode,
            case_insensitive: false,
            include_unchanged: false,
            ignore_whitespace: false,
        }
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set include unchanged
    pub fn include_unchanged(mut self, include: bool) -> Self {
        self.include_unchanged = include;
        self
    }

    /// Set ignore whitespace
    pub fn ignore_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_whitespace = ignore;
        self
    }
}

impl Default for ComparerConfig {
    fn default() -> Self {
        Self::new(CompareMode::Full)
    }
}
