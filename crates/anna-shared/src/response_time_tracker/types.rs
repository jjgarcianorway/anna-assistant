// v0.0.538: Response Time Tracker - Types (Phase 114)
// Enums and type definitions for response time tracking

use serde::{Deserialize, Serialize};

/// Response type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ResponseType {
    #[default]
    Direct,
    Recipe,
    Specialist,
    Escalated,
    Research,
    Clarification,
}

impl std::fmt::Display for ResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "Direct"),
            Self::Recipe => write!(f, "Recipe"),
            Self::Specialist => write!(f, "Specialist"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Research => write!(f, "Research"),
            Self::Clarification => write!(f, "Clarification"),
        }
    }
}

/// Complexity level of the response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ComplexityLevel {
    #[default]
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Simple"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Complex => write!(f, "Complex"),
            Self::VeryComplex => write!(f, "Very Complex"),
        }
    }
}

/// Time distribution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeDistribution {
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub std_dev: u64,
}
