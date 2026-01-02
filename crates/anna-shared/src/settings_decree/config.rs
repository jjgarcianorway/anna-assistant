// v0.0.720: Settings Decree - Config (Phase 296)
// Decree configuration

use serde::{Deserialize, Serialize};
use super::types::{DecreeType, DecreeBinding};

/// Decree config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeConfig {
    /// Name
    pub name: String,
    /// Decree type
    pub decree_type: DecreeType,
    /// Binding level
    pub binding: DecreeBinding,
    /// Max decrees
    pub max_decrees: usize,
}

impl DecreeConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            decree_type: DecreeType::Executive,
            binding: DecreeBinding::Mandatory,
            max_decrees: 100,
        }
    }

    /// Set type
    pub fn decree_type(mut self, dt: DecreeType) -> Self {
        self.decree_type = dt;
        self
    }

    /// Set binding
    pub fn binding(mut self, b: DecreeBinding) -> Self {
        self.binding = b;
        self
    }

    /// Set max decrees
    pub fn max_decrees(mut self, max: usize) -> Self {
        self.max_decrees = max;
        self
    }
}

impl Default for DecreeConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
