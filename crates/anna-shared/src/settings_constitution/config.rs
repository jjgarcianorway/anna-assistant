// v0.0.725: Constitution Config (Phase 301)

use serde::{Deserialize, Serialize};
use super::types::{ConstitutionType, ConstitutionBranch};

/// Constitution config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionConfig {
    /// Name
    pub name: String,
    /// Constitution type
    pub constitution_type: ConstitutionType,
    /// Branch
    pub branch: ConstitutionBranch,
    /// Max articles
    pub max_articles: usize,
}

impl ConstitutionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constitution_type: ConstitutionType::Written,
            branch: ConstitutionBranch::Executive,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn constitution_type(mut self, ct: ConstitutionType) -> Self {
        self.constitution_type = ct;
        self
    }

    /// Set branch
    pub fn branch(mut self, b: ConstitutionBranch) -> Self {
        self.branch = b;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConstitutionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
