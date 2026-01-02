// v0.0.713: Settings Notice Types (Phase 289)
// Notice type and priority enums

use serde::{Deserialize, Serialize};

/// Notice type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NoticeType {
    /// Information notice
    #[default]
    Information,
    /// Warning notice
    Warning,
    /// Alert notice
    Alert,
    /// Announcement notice
    Announcement,
}

impl std::fmt::Display for NoticeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Information => write!(f, "information"),
            Self::Warning => write!(f, "warning"),
            Self::Alert => write!(f, "alert"),
            Self::Announcement => write!(f, "announcement"),
        }
    }
}

/// Notice priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NoticePriority {
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

impl std::fmt::Display for NoticePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notice_type_display() {
        assert_eq!(format!("{}", NoticeType::Information), "information");
        assert_eq!(format!("{}", NoticeType::Alert), "alert");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", NoticePriority::Low), "low");
        assert_eq!(format!("{}", NoticePriority::Urgent), "urgent");
    }
}
