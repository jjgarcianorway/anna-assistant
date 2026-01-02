// v0.0.660: Settings Versioner - Version Types
// Enums and basic types for versioning

use serde::{Deserialize, Serialize};

/// Version scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VersionScheme {
    /// Semantic versioning (major.minor.patch)
    #[default]
    Semantic,
    /// Sequential numbering
    Sequential,
    /// Date-based
    DateBased,
    /// Hash-based
    HashBased,
}

impl std::fmt::Display for VersionScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semantic => write!(f, "semantic"),
            Self::Sequential => write!(f, "sequential"),
            Self::DateBased => write!(f, "date_based"),
            Self::HashBased => write!(f, "hash_based"),
        }
    }
}

/// Version bump type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BumpType {
    /// Major version bump
    Major,
    /// Minor version bump
    #[default]
    Minor,
    /// Patch version bump
    Patch,
    /// Auto-detect based on changes
    Auto,
}

impl std::fmt::Display for BumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::Auto => write!(f, "auto"),
        }
    }
}
