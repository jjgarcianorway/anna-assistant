// v0.0.717: Settings Circular - Types (Phase 293)
// Circular types and scopes

use serde::{Deserialize, Serialize};

/// Circular type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CircularType {
    /// Policy circular
    #[default]
    Policy,
    /// Information circular
    Information,
    /// Directive circular
    Directive,
    /// Advisory circular
    Advisory,
}

impl std::fmt::Display for CircularType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Policy => write!(f, "policy"),
            Self::Information => write!(f, "information"),
            Self::Directive => write!(f, "directive"),
            Self::Advisory => write!(f, "advisory"),
        }
    }
}

/// Circular scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CircularScope {
    /// All scope
    #[default]
    All,
    /// Department scope
    Department,
    /// Team scope
    Team,
    /// Individual scope
    Individual,
}

impl std::fmt::Display for CircularScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Department => write!(f, "department"),
            Self::Team => write!(f, "team"),
            Self::Individual => write!(f, "individual"),
        }
    }
}
