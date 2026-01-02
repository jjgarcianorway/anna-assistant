// v0.0.764: Settings Pasture - Types (Phase 340)

use serde::{Deserialize, Serialize};

/// Pasture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PastureType {
    /// Permanent pasture
    #[default]
    Permanent,
    /// Rotational pasture
    Rotational,
    /// Intensive pasture
    Intensive,
    /// Rough pasture
    Rough,
}

impl std::fmt::Display for PastureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Permanent => write!(f, "permanent"),
            Self::Rotational => write!(f, "rotational"),
            Self::Intensive => write!(f, "intensive"),
            Self::Rough => write!(f, "rough"),
        }
    }
}

/// Pasture status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PastureStatus {
    /// Open status
    #[default]
    Open,
    /// Grazed status
    Grazed,
    /// Rested status
    Rested,
    /// Improved status
    Improved,
}

impl std::fmt::Display for PastureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Grazed => write!(f, "grazed"),
            Self::Rested => write!(f, "rested"),
            Self::Improved => write!(f, "improved"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pasture_type_display() {
        assert_eq!(format!("{}", PastureType::Permanent), "permanent");
        assert_eq!(format!("{}", PastureType::Rotational), "rotational");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PastureStatus::Open), "open");
        assert_eq!(format!("{}", PastureStatus::Grazed), "grazed");
    }
}
