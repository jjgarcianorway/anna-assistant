// v0.0.639: Settings Notifier - Priority (Phase 215)
// Notification priority types

use serde::{Deserialize, Serialize};

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum NotifyPriority {
    /// Low priority
    Low,
    /// Normal priority
    #[default]
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
    /// Critical priority
    Critical,
}

impl std::fmt::Display for NotifyPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
            Self::Critical => write!(f, "critical"),
        }
    }
}
