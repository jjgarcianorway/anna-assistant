// v0.0.725: Settings Constitution Types (Phase 301)

use serde::{Deserialize, Serialize};

/// Constitution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConstitutionType {
    /// Written constitution
    #[default]
    Written,
    /// Unwritten constitution
    Unwritten,
    /// Codified constitution
    Codified,
    /// Uncodified constitution
    Uncodified,
}

impl std::fmt::Display for ConstitutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Written => write!(f, "written"),
            Self::Unwritten => write!(f, "unwritten"),
            Self::Codified => write!(f, "codified"),
            Self::Uncodified => write!(f, "uncodified"),
        }
    }
}

/// Constitution branch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConstitutionBranch {
    /// Executive branch
    #[default]
    Executive,
    /// Legislative branch
    Legislative,
    /// Judicial branch
    Judicial,
    /// Administrative branch
    Administrative,
}

impl std::fmt::Display for ConstitutionBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Legislative => write!(f, "legislative"),
            Self::Judicial => write!(f, "judicial"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}
