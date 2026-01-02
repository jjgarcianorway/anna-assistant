// v0.0.719: Settings Edict - Config
// Configuration for edict system

use serde::{Deserialize, Serialize};
use super::types::{EdictType, EdictStatus};

/// Edict config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdictConfig {
    /// Name
    pub name: String,
    /// Edict type
    pub edict_type: EdictType,
    /// Default status
    pub default_status: EdictStatus,
    /// Max edicts
    pub max_edicts: usize,
}

impl EdictConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            edict_type: EdictType::Royal,
            default_status: EdictStatus::Draft,
            max_edicts: 100,
        }
    }

    /// Set type
    pub fn edict_type(mut self, et: EdictType) -> Self {
        self.edict_type = et;
        self
    }

    /// Set default status
    pub fn default_status(mut self, ds: EdictStatus) -> Self {
        self.default_status = ds;
        self
    }

    /// Set max edicts
    pub fn max_edicts(mut self, max: usize) -> Self {
        self.max_edicts = max;
        self
    }
}

impl Default for EdictConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
