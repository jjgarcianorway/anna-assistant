// v0.0.734: Settings Entente (Phase 310)
// Entente configuration

use serde::{Deserialize, Serialize};
use super::types::{EntenteType, EntenteStatus};

/// Entente config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntenteConfig {
    /// Name
    pub name: String,
    /// Entente type
    pub entente_type: EntenteType,
    /// Status
    pub status: EntenteStatus,
    /// Max understandings
    pub max_understandings: usize,
}

impl EntenteConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entente_type: EntenteType::Cordiale,
            status: EntenteStatus::Informal,
            max_understandings: 100,
        }
    }

    /// Set type
    pub fn entente_type(mut self, et: EntenteType) -> Self {
        self.entente_type = et;
        self
    }

    /// Set status
    pub fn status(mut self, s: EntenteStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max understandings
    pub fn max_understandings(mut self, max: usize) -> Self {
        self.max_understandings = max;
        self
    }
}

impl Default for EntenteConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
