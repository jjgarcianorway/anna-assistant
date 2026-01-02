// v0.0.749: Settings County Types (Phase 325)
// County type and status enums

use serde::{Deserialize, Serialize};

/// County type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CountyType {
    /// Metropolitan county
    #[default]
    Metropolitan,
    /// Rural county
    Rural,
    /// Historic county
    Historic,
    /// Administrative county
    Administrative,
}

impl std::fmt::Display for CountyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metropolitan => write!(f, "metropolitan"),
            Self::Rural => write!(f, "rural"),
            Self::Historic => write!(f, "historic"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// County status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CountyStatus {
    /// Established status
    #[default]
    Established,
    /// Active status
    Active,
    /// Merged status
    Merged,
    /// Abolished status
    Abolished,
}

impl std::fmt::Display for CountyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Established => write!(f, "established"),
            Self::Active => write!(f, "active"),
            Self::Merged => write!(f, "merged"),
            Self::Abolished => write!(f, "abolished"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_county_type_display() {
        assert_eq!(format!("{}", CountyType::Metropolitan), "metropolitan");
        assert_eq!(format!("{}", CountyType::Rural), "rural");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CountyStatus::Established), "established");
        assert_eq!(format!("{}", CountyStatus::Active), "active");
    }
}
