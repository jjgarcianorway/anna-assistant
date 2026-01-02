// v0.0.673: Settings Selector Config (Phase 249)
// Configuration for settings selector

use serde::{Deserialize, Serialize};
use super::types::{SelectorType, MatchMode};

/// Selector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorConfig {
    /// Default selector type
    pub default_type: SelectorType,
    /// Default match mode
    pub default_match: MatchMode,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Max selections
    pub max_selections: usize,
}

impl SelectorConfig {
    /// Create new config
    pub fn new(selector_type: SelectorType) -> Self {
        Self {
            default_type: selector_type,
            default_match: MatchMode::Exact,
            case_insensitive: true,
            max_selections: 1000,
        }
    }

    /// Set match mode
    pub fn match_mode(mut self, mode: MatchMode) -> Self {
        self.default_match = mode;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set max selections
    pub fn max_selections(mut self, max: usize) -> Self {
        self.max_selections = max;
        self
    }
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self::new(SelectorType::Pattern)
    }
}
