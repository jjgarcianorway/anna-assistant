// v0.0.704: Gazette Configuration (Phase 280)

use serde::{Deserialize, Serialize};
use super::types::GazetteType;

/// Gazette config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteConfig {
    /// Name
    pub name: String,
    /// Gazette type
    pub gazette_type: GazetteType,
    /// Issue number
    pub issue_number: usize,
    /// Max notices
    pub max_notices: usize,
}

impl GazetteConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            gazette_type: GazetteType::Official,
            issue_number: 1,
            max_notices: 100,
        }
    }

    /// Set type
    pub fn gazette_type(mut self, gt: GazetteType) -> Self {
        self.gazette_type = gt;
        self
    }

    /// Set issue number
    pub fn issue_number(mut self, num: usize) -> Self {
        self.issue_number = num;
        self
    }

    /// Set max notices
    pub fn max_notices(mut self, max: usize) -> Self {
        self.max_notices = max;
        self
    }
}

impl Default for GazetteConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = GazetteConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GazetteConfig::new("test")
            .gazette_type(GazetteType::Special)
            .issue_number(5);
        assert_eq!(c.gazette_type, GazetteType::Special);
        assert_eq!(c.issue_number, 5);
    }
}
