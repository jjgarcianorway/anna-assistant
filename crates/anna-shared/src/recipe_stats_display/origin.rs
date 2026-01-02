//! Recipe origin tracking.

use serde::{Deserialize, Serialize};

/// Recipe origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecipeOriginType {
    /// Built-in seed recipe
    Seed,
    /// Learned from specialist
    Learned,
    /// Learned from user interaction
    UserTaught,
    /// Imported from file
    Imported,
}

impl RecipeOriginType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Seed => "Built-in",
            Self::Learned => "Learned",
            Self::UserTaught => "User Taught",
            Self::Imported => "Imported",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_display() {
        assert_eq!(RecipeOriginType::Seed.display_name(), "Built-in");
        assert_eq!(RecipeOriginType::Learned.display_name(), "Learned");
    }
}
