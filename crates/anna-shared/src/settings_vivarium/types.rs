use serde::{Deserialize, Serialize};

/// Vivarium type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VivariumType {
    /// Reptile vivarium
    #[default]
    Reptile,
    /// Amphibian vivarium
    Amphibian,
    /// Invertebrate vivarium
    Invertebrate,
    /// Mixed vivarium
    Mixed,
}

impl std::fmt::Display for VivariumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reptile => write!(f, "reptile"),
            Self::Amphibian => write!(f, "amphibian"),
            Self::Invertebrate => write!(f, "invertebrate"),
            Self::Mixed => write!(f, "mixed"),
        }
    }
}

/// Vivarium status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VivariumStatus {
    /// Setup status
    #[default]
    Setup,
    /// Established status
    Established,
    /// Breeding status
    Breeding,
    /// Resting status
    Resting,
}

impl std::fmt::Display for VivariumStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup => write!(f, "setup"),
            Self::Established => write!(f, "established"),
            Self::Breeding => write!(f, "breeding"),
            Self::Resting => write!(f, "resting"),
        }
    }
}
