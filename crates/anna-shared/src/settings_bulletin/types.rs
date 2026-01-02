// v0.0.706: Settings Bulletin - Types (Phase 282)
// Bulletin types and priorities

use serde::{Deserialize, Serialize};

/// Bulletin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BulletinType {
    /// News bulletin
    #[default]
    News,
    /// Alert bulletin
    Alert,
    /// Update bulletin
    Update,
    /// Archive bulletin
    Archive,
}

impl std::fmt::Display for BulletinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::News => write!(f, "news"),
            Self::Alert => write!(f, "alert"),
            Self::Update => write!(f, "update"),
            Self::Archive => write!(f, "archive"),
        }
    }
}

/// Bulletin priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BulletinPriority {
    /// Low priority
    #[default]
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

impl std::fmt::Display for BulletinPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
        }
    }
}