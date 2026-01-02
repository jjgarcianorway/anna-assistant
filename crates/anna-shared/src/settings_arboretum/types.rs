// v0.0.772: Settings Arboretum Types (Phase 348)
// Core types for arboretum classification

use serde::{Deserialize, Serialize};

/// Arboretum type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArboretumType {
    /// Public arboretum
    #[default]
    Public,
    /// University arboretum
    University,
    /// Memorial arboretum
    Memorial,
    /// Research arboretum
    Research,
}

impl std::fmt::Display for ArboretumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::University => write!(f, "university"),
            Self::Memorial => write!(f, "memorial"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Arboretum status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArboretumStatus {
    /// Open status
    #[default]
    Open,
    /// Planting status
    Planting,
    /// Surveying status
    Surveying,
    /// Closed status
    Closed,
}

impl std::fmt::Display for ArboretumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Planting => write!(f, "planting"),
            Self::Surveying => write!(f, "surveying"),
            Self::Closed => write!(f, "closed"),
        }
    }
}
