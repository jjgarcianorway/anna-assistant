// v0.0.704: Settings Gazette Types (Phase 280)

use serde::{Deserialize, Serialize};

/// Gazette type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GazetteType {
    /// Official gazette
    #[default]
    Official,
    /// Weekly gazette
    Weekly,
    /// Special gazette
    Special,
    /// Extraordinary gazette
    Extraordinary,
}

impl std::fmt::Display for GazetteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Official => write!(f, "official"),
            Self::Weekly => write!(f, "weekly"),
            Self::Special => write!(f, "special"),
            Self::Extraordinary => write!(f, "extraordinary"),
        }
    }
}

/// Gazette status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GazetteStatus {
    /// Draft
    #[default]
    Draft,
    /// Review
    Review,
    /// Published
    Published,
    /// Superseded
    Superseded,
}

impl std::fmt::Display for GazetteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Review => write!(f, "review"),
            Self::Published => write!(f, "published"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gazette_type_display() {
        assert_eq!(format!("{}", GazetteType::Official), "official");
        assert_eq!(format!("{}", GazetteType::Special), "special");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GazetteStatus::Draft), "draft");
        assert_eq!(format!("{}", GazetteStatus::Published), "published");
    }
}
