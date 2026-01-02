// v0.0.765: Settings Grove (Phase 341)
// Grove types and enums

use serde::{Deserialize, Serialize};

/// Grove type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroveType {
    /// Oak grove
    #[default]
    Oak,
    /// Olive grove
    Olive,
    /// Citrus grove
    Citrus,
    /// Sacred grove
    Sacred,
}

impl std::fmt::Display for GroveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oak => write!(f, "oak"),
            Self::Olive => write!(f, "olive"),
            Self::Citrus => write!(f, "citrus"),
            Self::Sacred => write!(f, "sacred"),
        }
    }
}

/// Grove status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroveStatus {
    /// Planted status
    #[default]
    Planted,
    /// Maturing status
    Maturing,
    /// Productive status
    Productive,
    /// Resting status
    Resting,
}

impl std::fmt::Display for GroveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planted => write!(f, "planted"),
            Self::Maturing => write!(f, "maturing"),
            Self::Productive => write!(f, "productive"),
            Self::Resting => write!(f, "resting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grove_type_display() {
        assert_eq!(format!("{}", GroveType::Oak), "oak");
        assert_eq!(format!("{}", GroveType::Olive), "olive");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GroveStatus::Planted), "planted");
        assert_eq!(format!("{}", GroveStatus::Productive), "productive");
    }
}
