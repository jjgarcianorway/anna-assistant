// v0.0.765: Settings Grove (Phase 341)
// Grove configuration

use serde::{Deserialize, Serialize};
use super::types::{GroveType, GroveStatus};

/// Grove config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroveConfig {
    /// Name
    pub name: String,
    /// Grove type
    pub grove_type: GroveType,
    /// Status
    pub status: GroveStatus,
    /// Max trees
    pub max_trees: usize,
}

impl GroveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            grove_type: GroveType::Oak,
            status: GroveStatus::Planted,
            max_trees: 100,
        }
    }

    /// Set type
    pub fn grove_type(mut self, gt: GroveType) -> Self {
        self.grove_type = gt;
        self
    }

    /// Set status
    pub fn status(mut self, s: GroveStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max trees
    pub fn max_trees(mut self, max: usize) -> Self {
        self.max_trees = max;
        self
    }
}

impl Default for GroveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = GroveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GroveConfig::new("test")
            .grove_type(GroveType::Citrus)
            .status(GroveStatus::Maturing);
        assert_eq!(c.grove_type, GroveType::Citrus);
        assert_eq!(c.status, GroveStatus::Maturing);
    }
}
