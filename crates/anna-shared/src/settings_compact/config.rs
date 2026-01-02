// v0.0.729: Settings Compact (Phase 305)
// Compact configuration

use serde::{Deserialize, Serialize};
use super::types::{CompactType, CompactStatus};

/// Compact config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConfig {
    /// Name
    pub name: String,
    /// Compact type
    pub compact_type: CompactType,
    /// Status
    pub status: CompactStatus,
    /// Max terms
    pub max_terms: usize,
}

impl CompactConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compact_type: CompactType::Interstate,
            status: CompactStatus::Proposed,
            max_terms: 100,
        }
    }

    /// Set type
    pub fn compact_type(mut self, ct: CompactType) -> Self {
        self.compact_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CompactStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max terms
    pub fn max_terms(mut self, max: usize) -> Self {
        self.max_terms = max;
        self
    }
}

impl Default for CompactConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = CompactConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CompactConfig::new("test")
            .compact_type(CompactType::Regional)
            .status(CompactStatus::Negotiating);
        assert_eq!(c.compact_type, CompactType::Regional);
        assert_eq!(c.status, CompactStatus::Negotiating);
    }
}
