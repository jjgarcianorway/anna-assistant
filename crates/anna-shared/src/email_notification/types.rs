//! Email notification types and core data structures

use serde::{Deserialize, Serialize};

/// Email notification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Cancelled,
    Queued,
}

impl NotificationStatus {
    pub fn name(&self) -> &'static str {
        match self {
            NotificationStatus::Pending => "Pending",
            NotificationStatus::Sent => "Sent",
            NotificationStatus::Failed => "Failed",
            NotificationStatus::Cancelled => "Cancelled",
            NotificationStatus::Queued => "Queued",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            NotificationStatus::Pending => "...",
            NotificationStatus::Sent => "✓",
            NotificationStatus::Failed => "✗",
            NotificationStatus::Cancelled => "-",
            NotificationStatus::Queued => "~",
        }
    }
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationType {
    TaskComplete,
    TaskFailed,
    ResearchResult,
    TicketUpdate,
    SystemAlert,
    ScheduledReport,
    Custom,
}

impl NotificationType {
    pub fn name(&self) -> &'static str {
        match self {
            NotificationType::TaskComplete => "Task Complete",
            NotificationType::TaskFailed => "Task Failed",
            NotificationType::ResearchResult => "Research Result",
            NotificationType::TicketUpdate => "Ticket Update",
            NotificationType::SystemAlert => "System Alert",
            NotificationType::ScheduledReport => "Scheduled Report",
            NotificationType::Custom => "Custom",
        }
    }
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// User's email address
    pub address: String,
    /// Whether user consented to notifications
    pub consent_given: bool,
    /// When consent was given
    pub consent_timestamp: Option<u64>,
    /// Preferred notification types
    pub enabled_types: Vec<NotificationType>,
    /// Do not disturb hours (start, end in 24h format)
    pub dnd_hours: Option<(u8, u8)>,
    /// Maximum emails per day
    pub daily_limit: Option<u32>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            address: String::new(),
            consent_given: false,
            consent_timestamp: None,
            enabled_types: vec![
                NotificationType::TaskComplete,
                NotificationType::TaskFailed,
                NotificationType::ResearchResult,
            ],
            dnd_hours: None,
            daily_limit: Some(10),
        }
    }
}

/// A notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// Unique notification ID
    pub id: String,
    /// Notification type
    pub notification_type: NotificationType,
    /// Status
    pub status: NotificationStatus,
    /// Subject line
    pub subject: String,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// When created
    pub created_at: u64,
    /// When sent (if sent)
    pub sent_at: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
    /// Retry count
    pub retry_count: u32,
}
