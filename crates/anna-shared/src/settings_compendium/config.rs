// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Compendium configuration

use serde::{Deserialize, Serialize};
use super::types::{CompendiumType, CompendiumEdition};

/// Compendium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumConfig {
    /// Name
    pub name: String,
    /// Compendium type
    pub compendium_type: CompendiumType,
    /// Edition
    pub edition: CompendiumEdition,
    /// Max volumes
    pub max_volumes: usize,
}

impl CompendiumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compendium_type: CompendiumType::Reference,
            edition: CompendiumEdition::First,
            max_volumes: 100,
        }
    }

    /// Set type
    pub fn compendium_type(mut self, ct: CompendiumType) -> Self {
        self.compendium_type = ct;
        self
    }

    /// Set edition
    pub fn edition(mut self, ed: CompendiumEdition) -> Self {
        self.edition = ed;
        self
    }

    /// Set max volumes
    pub fn max_volumes(mut self, max: usize) -> Self {
        self.max_volumes = max;
        self
    }
}

impl Default for CompendiumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
