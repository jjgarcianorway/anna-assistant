// v0.0.747: Settings Region Types (Phase 323)
// Region type and status enums

use serde::{Deserialize, Serialize};

/// Region type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RegionType {
    /// Administrative region
    #[default]
    Administrative,
    /// Economic region
    Economic,
    /// Cultural region
    Cultural,
    /// Geographic region
    Geographic,
}

impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Administrative => write!(f, "administrative"),
            Self::Economic => write!(f, "economic"),
            Self::Cultural => write!(f, "cultural"),
            Self::Geographic => write!(f, "geographic"),
        }
    }
}

/// Region status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RegionStatus {
    /// Defined status
    #[default]
    Defined,
    /// Active status
    Active,
    /// Expanding status
    Expanding,
    /// Contracting status
    Contracting,
}

impl std::fmt::Display for RegionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined => write!(f, "defined"),
            Self::Active => write!(f, "active"),
            Self::Expanding => write!(f, "expanding"),
            Self::Contracting => write!(f, "contracting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_type_display() {
        assert_eq!(format!("{}", RegionType::Administrative), "administrative");
        assert_eq!(format!("{}", RegionType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", RegionStatus::Defined), "defined");
        assert_eq!(format!("{}", RegionStatus::Active), "active");
    }
}
