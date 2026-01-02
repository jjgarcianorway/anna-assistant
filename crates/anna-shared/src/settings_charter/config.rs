// v0.0.724: Settings Charter - Config module
// Charter configuration

use serde::{Deserialize, Serialize};
use super::types::{CharterType, CharterStatus};

/// Charter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterConfig {
    /// Name
    pub name: String,
    /// Charter type
    pub charter_type: CharterType,
    /// Status
    pub status: CharterStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl CharterConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            charter_type: CharterType::Founding,
            status: CharterStatus::Draft,
            max_provisions: 150,
        }
    }

    /// Set type
    pub fn charter_type(mut self, ct: CharterType) -> Self {
        self.charter_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CharterStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for CharterConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
