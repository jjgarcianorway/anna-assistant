// v0.0.778: Settings Aviary (Phase 354)
// Bird aviary types

use serde::{Deserialize, Serialize};

/// Aviary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AviaryType {
    /// Flight aviary
    #[default]
    Flight,
    /// Breeding aviary
    Breeding,
    /// Display aviary
    Display,
    /// Rescue aviary
    Rescue,
}

impl std::fmt::Display for AviaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flight => write!(f, "flight"),
            Self::Breeding => write!(f, "breeding"),
            Self::Display => write!(f, "display"),
            Self::Rescue => write!(f, "rescue"),
        }
    }
}

/// Aviary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AviaryStatus {
    /// Active status
    #[default]
    Active,
    /// Nesting status
    Nesting,
    /// Molting status
    Molting,
    /// Quarantine status
    Quarantine,
}

impl std::fmt::Display for AviaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Nesting => write!(f, "nesting"),
            Self::Molting => write!(f, "molting"),
            Self::Quarantine => write!(f, "quarantine"),
        }
    }
}
