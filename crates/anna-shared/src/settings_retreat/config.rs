// v0.0.785: Settings Retreat - Config (Phase 361)

use serde::{Deserialize, Serialize};
use super::types::{RetreatType, RetreatStatus};

/// Retreat config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetreatConfig {
    /// Name
    pub name: String,
    /// Retreat type
    pub retreat_type: RetreatType,
    /// Status
    pub status: RetreatStatus,
    /// Max visitors
    pub max_visitors: usize,
}

impl RetreatConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            retreat_type: RetreatType::Peaceful,
            status: RetreatStatus::Open,
            max_visitors: 100,
        }
    }

    /// Set type
    pub fn retreat_type(mut self, rt: RetreatType) -> Self {
        self.retreat_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RetreatStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max visitors
    pub fn max_visitors(mut self, max: usize) -> Self {
        self.max_visitors = max;
        self
    }
}

impl Default for RetreatConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = RetreatConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RetreatConfig::new("test")
            .retreat_type(RetreatType::Mountain)
            .status(RetreatStatus::Meditating);
        assert_eq!(c.retreat_type, RetreatType::Mountain);
        assert_eq!(c.status, RetreatStatus::Meditating);
    }
}
