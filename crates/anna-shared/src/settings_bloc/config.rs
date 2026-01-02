// v0.0.740: Settings Bloc Config (Phase 316)
// Bloc configuration

use serde::{Deserialize, Serialize};
use super::types::{BlocType, BlocStatus};

/// Bloc config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocConfig {
    /// Name
    pub name: String,
    /// Bloc type
    pub bloc_type: BlocType,
    /// Status
    pub status: BlocStatus,
    /// Max policies
    pub max_policies: usize,
}

impl BlocConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bloc_type: BlocType::Trading,
            status: BlocStatus::Forming,
            max_policies: 100,
        }
    }

    /// Set type
    pub fn bloc_type(mut self, bt: BlocType) -> Self {
        self.bloc_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BlocStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max policies
    pub fn max_policies(mut self, max: usize) -> Self {
        self.max_policies = max;
        self
    }
}

impl Default for BlocConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
