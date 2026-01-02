// v0.0.663: Settings Graph - Link Types
// Core link type definitions for settings graph

use serde::{Deserialize, Serialize};

/// Link type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LinkType {
    /// Reference link
    #[default]
    Reference,
    /// Alias link
    Alias,
    /// Dependency link
    Dependency,
    /// Override link
    Override,
    /// Computed link
    Computed,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference => write!(f, "reference"),
            Self::Alias => write!(f, "alias"),
            Self::Dependency => write!(f, "dependency"),
            Self::Override => write!(f, "override"),
            Self::Computed => write!(f, "computed"),
        }
    }
}

/// Link direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinkDirection {
    /// Unidirectional (source -> target)
    #[default]
    Unidirectional,
    /// Bidirectional (source <-> target)
    Bidirectional,
    /// Reverse (target -> source)
    Reverse,
}

impl std::fmt::Display for LinkDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unidirectional => write!(f, "unidirectional"),
            Self::Bidirectional => write!(f, "bidirectional"),
            Self::Reverse => write!(f, "reverse"),
        }
    }
}
