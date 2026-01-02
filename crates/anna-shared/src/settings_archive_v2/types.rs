// v0.0.702: Settings Archive V2 (Phase 278)
// Archive types and retention enums

use serde::{Deserialize, Serialize};

/// Archive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArchiveTypeV2 {
    /// Cold archive
    #[default]
    Cold,
    /// Warm archive
    Warm,
    /// Deep archive
    Deep,
    /// Glacier archive
    Glacier,
}

impl std::fmt::Display for ArchiveTypeV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cold => write!(f, "cold"),
            Self::Warm => write!(f, "warm"),
            Self::Deep => write!(f, "deep"),
            Self::Glacier => write!(f, "glacier"),
        }
    }
}

/// Archive retention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArchiveRetention {
    /// 30 days
    #[default]
    Days30,
    /// 90 days
    Days90,
    /// 1 year
    Year1,
    /// Indefinite
    Indefinite,
}

impl std::fmt::Display for ArchiveRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Days30 => write!(f, "30d"),
            Self::Days90 => write!(f, "90d"),
            Self::Year1 => write!(f, "1y"),
            Self::Indefinite => write!(f, "indefinite"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_type_display() {
        assert_eq!(format!("{}", ArchiveTypeV2::Cold), "cold");
        assert_eq!(format!("{}", ArchiveTypeV2::Glacier), "glacier");
    }

    #[test]
    fn test_retention_display() {
        assert_eq!(format!("{}", ArchiveRetention::Days30), "30d");
        assert_eq!(format!("{}", ArchiveRetention::Indefinite), "indefinite");
    }
}
