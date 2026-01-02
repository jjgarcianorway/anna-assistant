// v0.0.641: Settings Inspector Types (Phase 217)
// Basic types and enums for settings inspection

use serde::{Deserialize, Serialize};

/// Inspection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum InspectionType {
    /// Structure inspection
    #[default]
    Structure,
    /// Value inspection
    Value,
    /// Type inspection
    Type,
    /// Dependency inspection
    Dependency,
    /// Full inspection
    Full,
}

impl std::fmt::Display for InspectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structure => write!(f, "structure"),
            Self::Value => write!(f, "value"),
            Self::Type => write!(f, "type"),
            Self::Dependency => write!(f, "dependency"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Inspection depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum InspectionDepth {
    /// Shallow
    Shallow,
    /// Normal
    #[default]
    Normal,
    /// Deep
    Deep,
    /// Complete
    Complete,
}

impl std::fmt::Display for InspectionDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shallow => write!(f, "shallow"),
            Self::Normal => write!(f, "normal"),
            Self::Deep => write!(f, "deep"),
            Self::Complete => write!(f, "complete"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspection_type_display() {
        assert_eq!(format!("{}", InspectionType::Structure), "structure");
        assert_eq!(format!("{}", InspectionType::Value), "value");
    }

    #[test]
    fn test_depth_display() {
        assert_eq!(format!("{}", InspectionDepth::Normal), "normal");
        assert_eq!(format!("{}", InspectionDepth::Deep), "deep");
    }
}
