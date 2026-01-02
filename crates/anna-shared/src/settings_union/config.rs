// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Config

use serde::{Deserialize, Serialize};
use super::types::{UnionType, UnionStatus};

/// Union config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionConfig {
    /// Name
    pub name: String,
    /// Union type
    pub union_type: UnionType,
    /// Status
    pub status: UnionStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl UnionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            union_type: UnionType::Full,
            status: UnionStatus::Proposed,
            max_provisions: 100,
        }
    }

    /// Set type
    pub fn union_type(mut self, ut: UnionType) -> Self {
        self.union_type = ut;
        self
    }

    /// Set status
    pub fn status(mut self, s: UnionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for UnionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
