// v0.0.746: Settings Province - Types (Phase 322)
// Province types and status enums

use serde::{Deserialize, Serialize};

/// Province type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProvinceType {
    /// Autonomous province
    #[default]
    Autonomous,
    /// Imperial province
    Imperial,
    /// Colonial province
    Colonial,
    /// Federal province
    Federal,
}

impl std::fmt::Display for ProvinceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Autonomous => write!(f, "autonomous"),
            Self::Imperial => write!(f, "imperial"),
            Self::Colonial => write!(f, "colonial"),
            Self::Federal => write!(f, "federal"),
        }
    }
}

/// Province status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProvinceStatus {
    /// Established status
    #[default]
    Established,
    /// Developing status
    Developing,
    /// Integrated status
    Integrated,
    /// Reorganizing status
    Reorganizing,
}

impl std::fmt::Display for ProvinceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Established => write!(f, "established"),
            Self::Developing => write!(f, "developing"),
            Self::Integrated => write!(f, "integrated"),
            Self::Reorganizing => write!(f, "reorganizing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_province_type_display() {
        assert_eq!(format!("{}", ProvinceType::Autonomous), "autonomous");
        assert_eq!(format!("{}", ProvinceType::Imperial), "imperial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ProvinceStatus::Established), "established");
        assert_eq!(format!("{}", ProvinceStatus::Integrated), "integrated");
    }
}
