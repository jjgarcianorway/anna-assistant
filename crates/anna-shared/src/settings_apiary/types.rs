// v0.0.779: Settings Apiary - Types (Phase 355)
// Apiary type and status enums

use serde::{Deserialize, Serialize};

/// Apiary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ApiaryType {
    /// Honey apiary
    #[default]
    Honey,
    /// Pollination apiary
    Pollination,
    /// Queen apiary
    Queen,
    /// Research apiary
    Research,
}

impl std::fmt::Display for ApiaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Honey => write!(f, "honey"),
            Self::Pollination => write!(f, "pollination"),
            Self::Queen => write!(f, "queen"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Apiary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ApiaryStatus {
    /// Active status
    #[default]
    Active,
    /// Swarming status
    Swarming,
    /// Harvesting status
    Harvesting,
    /// Wintering status
    Wintering,
}

impl std::fmt::Display for ApiaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Swarming => write!(f, "swarming"),
            Self::Harvesting => write!(f, "harvesting"),
            Self::Wintering => write!(f, "wintering"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apiary_type_display() {
        assert_eq!(format!("{}", ApiaryType::Honey), "honey");
        assert_eq!(format!("{}", ApiaryType::Pollination), "pollination");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ApiaryStatus::Active), "active");
        assert_eq!(format!("{}", ApiaryStatus::Harvesting), "harvesting");
    }
}
