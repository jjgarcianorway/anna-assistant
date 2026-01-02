// v0.0.533: Notification Types (Phase 109)
// Notification channel, priority, and status enums

use serde::{Deserialize, Serialize};

/// Notification channel type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Libnotify,
    Wall,
    Terminal,
    Log,
}

impl Default for NotificationChannel {
    fn default() -> Self {
        Self::Terminal
    }
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "Email"),
            Self::Libnotify => write!(f, "Desktop Notification"),
            Self::Wall => write!(f, "Wall Message"),
            Self::Terminal => write!(f, "Terminal"),
            Self::Log => write!(f, "Log"),
        }
    }
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Urgent => write!(f, "Urgent"),
        }
    }
}

/// Notification delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DeliveryStatus {
    #[default]
    Pending,
    Sent,
    Delivered,
    Failed,
    Suppressed,
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Sent => write!(f, "Sent"),
            Self::Delivered => write!(f, "Delivered"),
            Self::Failed => write!(f, "Failed"),
            Self::Suppressed => write!(f, "Suppressed"),
        }
    }
}
