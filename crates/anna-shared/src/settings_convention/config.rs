// v0.0.733: Settings Convention Config (Phase 309)
// Configuration for conventions

use serde::{Deserialize, Serialize};
use super::types::{ConventionType, ConventionStatus};

/// Convention config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionConfig {
    /// Name
    pub name: String,
    /// Convention type
    pub convention_type: ConventionType,
    /// Status
    pub status: ConventionStatus,
    /// Max articles
    pub max_articles: usize,
}

impl ConventionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            convention_type: ConventionType::International,
            status: ConventionStatus::Draft,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn convention_type(mut self, ct: ConventionType) -> Self {
        self.convention_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConventionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConventionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ConventionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConventionConfig::new("test")
            .convention_type(ConventionType::Constitutional)
            .status(ConventionStatus::Adopted);
        assert_eq!(c.convention_type, ConventionType::Constitutional);
        assert_eq!(c.status, ConventionStatus::Adopted);
    }
}
