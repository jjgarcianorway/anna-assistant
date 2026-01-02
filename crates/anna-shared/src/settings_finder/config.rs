// v0.0.685: Finder Configuration (Phase 261)
// Configuration for settings finder

use serde::{Deserialize, Serialize};
use super::types::{FindMode, FindLimit};

/// Finder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinderConfig {
    /// Find mode
    pub mode: FindMode,
    /// Find limit
    pub limit: FindLimit,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Include partial matches
    pub partial_match: bool,
}

impl FinderConfig {
    /// Create new config
    pub fn new(mode: FindMode) -> Self {
        Self {
            mode,
            limit: FindLimit::All,
            case_insensitive: true,
            partial_match: true,
        }
    }

    /// Set limit
    pub fn limit(mut self, limit: FindLimit) -> Self {
        self.limit = limit;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set partial match
    pub fn partial_match(mut self, partial: bool) -> Self {
        self.partial_match = partial;
        self
    }
}

impl Default for FinderConfig {
    fn default() -> Self {
        Self::new(FindMode::KeyPattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = FinderConfig::new(FindMode::ExactKey);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = FinderConfig::new(FindMode::ValuePattern)
            .limit(FindLimit::Max(10))
            .case_insensitive(false);
        assert!(!c.case_insensitive);
    }
}
