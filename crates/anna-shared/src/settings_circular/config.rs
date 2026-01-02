// v0.0.717: Settings Circular - Config (Phase 293)
// Circular configuration

use serde::{Deserialize, Serialize};
use super::types::{CircularType, CircularScope};

/// Circular config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularConfig {
    /// Name
    pub name: String,
    /// Circular type
    pub circular_type: CircularType,
    /// Scope
    pub scope: CircularScope,
    /// Max circulars
    pub max_circulars: usize,
}

impl CircularConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            circular_type: CircularType::Policy,
            scope: CircularScope::All,
            max_circulars: 200,
        }
    }

    /// Set type
    pub fn circular_type(mut self, ct: CircularType) -> Self {
        self.circular_type = ct;
        self
    }

    /// Set scope
    pub fn scope(mut self, s: CircularScope) -> Self {
        self.scope = s;
        self
    }

    /// Set max circulars
    pub fn max_circulars(mut self, max: usize) -> Self {
        self.max_circulars = max;
        self
    }
}

impl Default for CircularConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
