//! Tier level enum (v0.0.182).

use serde::{Deserialize, Serialize};

/// Tier level for team members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Junior,
    Senior,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
        }
    }
}
