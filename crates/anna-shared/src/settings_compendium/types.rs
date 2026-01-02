// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Compendium types

use serde::{Deserialize, Serialize};

/// Compendium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompendiumType {
    /// Reference compendium
    #[default]
    Reference,
    /// Tutorial compendium
    Tutorial,
    /// Encyclopedia compendium
    Encyclopedia,
    /// Handbook compendium
    Handbook,
}

impl std::fmt::Display for CompendiumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference => write!(f, "reference"),
            Self::Tutorial => write!(f, "tutorial"),
            Self::Encyclopedia => write!(f, "encyclopedia"),
            Self::Handbook => write!(f, "handbook"),
        }
    }
}

/// Compendium edition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CompendiumEdition {
    /// First edition
    #[default]
    First,
    /// Revised edition
    Revised,
    /// Extended edition
    Extended,
    /// Final edition
    Final,
}

impl std::fmt::Display for CompendiumEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::Revised => write!(f, "revised"),
            Self::Extended => write!(f, "extended"),
            Self::Final => write!(f, "final"),
        }
    }
}
