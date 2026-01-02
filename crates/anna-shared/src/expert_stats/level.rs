//! Expert level definitions.

use serde::{Deserialize, Serialize};

/// Expert level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpertLevel {
    /// Junior specialist
    Junior,
    /// Senior specialist
    Senior,
}

impl ExpertLevel {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Junior => "Junior",
            Self::Senior => "Senior",
        }
    }

    /// Get short name
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Junior => "Jr",
            Self::Senior => "Sr",
        }
    }
}
