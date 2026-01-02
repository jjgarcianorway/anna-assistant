// v0.0.743: Settings Domain - Domain Config (Phase 319)
// Configuration for domain settings

use serde::{Deserialize, Serialize};
use super::domain_types::{DomainType, DomainStatus};

/// Domain config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Name
    pub name: String,
    /// Domain type
    pub domain_type: DomainType,
    /// Status
    pub status: DomainStatus,
    /// Max rights
    pub max_rights: usize,
}

impl DomainConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            domain_type: DomainType::Public,
            status: DomainStatus::Claimed,
            max_rights: 100,
        }
    }

    /// Set type
    pub fn domain_type(mut self, dt: DomainType) -> Self {
        self.domain_type = dt;
        self
    }

    /// Set status
    pub fn status(mut self, s: DomainStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max rights
    pub fn max_rights(mut self, max: usize) -> Self {
        self.max_rights = max;
        self
    }
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
