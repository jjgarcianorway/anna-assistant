// v0.0.708: Settings Memo (Phase 284)
// Core types and enums

use serde::{Deserialize, Serialize};

/// Memo type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MemoType {
    /// Internal memo
    #[default]
    Internal,
    /// External memo
    External,
    /// Confidential memo
    Confidential,
    /// Broadcast memo
    Broadcast,
}

impl std::fmt::Display for MemoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "internal"),
            Self::External => write!(f, "external"),
            Self::Confidential => write!(f, "confidential"),
            Self::Broadcast => write!(f, "broadcast"),
        }
    }
}

/// Memo status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MemoStatus {
    /// Draft
    #[default]
    Draft,
    /// Sent
    Sent,
    /// Read
    Read,
    /// Archived
    Archived,
}

impl std::fmt::Display for MemoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Sent => write!(f, "sent"),
            Self::Read => write!(f, "read"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_type_display() {
        assert_eq!(format!("{}", MemoType::Internal), "internal");
        assert_eq!(format!("{}", MemoType::Confidential), "confidential");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", MemoStatus::Draft), "draft");
        assert_eq!(format!("{}", MemoStatus::Sent), "sent");
    }
}
