// v0.0.738: Settings Confederation Config
// Configuration for confederation

use serde::{Deserialize, Serialize};
use super::types::{ConfederationType, ConfederationStatus};

/// Confederation config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfederationConfig {
    /// Name
    pub name: String,
    /// Confederation type
    pub confederation_type: ConfederationType,
    /// Status
    pub status: ConfederationStatus,
    /// Max articles
    pub max_articles: usize,
}

impl ConfederationConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            confederation_type: ConfederationType::Sovereign,
            status: ConfederationStatus::Forming,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn confederation_type(mut self, ct: ConfederationType) -> Self {
        self.confederation_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConfederationStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConfederationConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ConfederationConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConfederationConfig::new("test")
            .confederation_type(ConfederationType::Economic)
            .status(ConfederationStatus::Functional);
        assert_eq!(c.confederation_type, ConfederationType::Economic);
        assert_eq!(c.status, ConfederationStatus::Functional);
    }
}
