// v0.0.695: Settings Folio (Phase 271)
// Folio types and enums

use serde::{Deserialize, Serialize};

/// Folio type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FolioType {
    /// Active folio
    #[default]
    Active,
    /// Archived folio
    Archived,
    /// Template folio
    Template,
    /// Backup folio
    Backup,
}

impl std::fmt::Display for FolioType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::Template => write!(f, "template"),
            Self::Backup => write!(f, "backup"),
        }
    }
}

/// Folio status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FolioStatus {
    /// Open
    #[default]
    Open,
    /// Closed
    Closed,
    /// Locked
    Locked,
    /// Pending
    Pending,
}

impl std::fmt::Display for FolioStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "closed"),
            Self::Locked => write!(f, "locked"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folio_type_display() {
        assert_eq!(format!("{}", FolioType::Active), "active");
        assert_eq!(format!("{}", FolioType::Template), "template");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", FolioStatus::Open), "open");
        assert_eq!(format!("{}", FolioStatus::Locked), "locked");
    }
}
