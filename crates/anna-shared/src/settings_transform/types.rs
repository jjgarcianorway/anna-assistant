// v0.0.666: Settings Transform Types (Phase 242)
// Core type definitions for settings transformation

use serde::{Deserialize, Serialize};

/// Transform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TransformType {
    /// Map transformation
    #[default]
    Map,
    /// Filter transformation
    Filter,
    /// Reduce transformation
    Reduce,
    /// Flatten transformation
    Flatten,
    /// Group transformation
    Group,
}

impl std::fmt::Display for TransformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Map => write!(f, "map"),
            Self::Filter => write!(f, "filter"),
            Self::Reduce => write!(f, "reduce"),
            Self::Flatten => write!(f, "flatten"),
            Self::Group => write!(f, "group"),
        }
    }
}

/// Transform direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransformDirection {
    /// Forward transformation
    #[default]
    Forward,
    /// Reverse transformation
    Reverse,
    /// Bidirectional
    Bidirectional,
}

impl std::fmt::Display for TransformDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward => write!(f, "forward"),
            Self::Reverse => write!(f, "reverse"),
            Self::Bidirectional => write!(f, "bidirectional"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_type_display() {
        assert_eq!(format!("{}", TransformType::Map), "map");
        assert_eq!(format!("{}", TransformType::Filter), "filter");
    }

    #[test]
    fn test_direction_display() {
        assert_eq!(format!("{}", TransformDirection::Forward), "forward");
        assert_eq!(format!("{}", TransformDirection::Reverse), "reverse");
    }
}
