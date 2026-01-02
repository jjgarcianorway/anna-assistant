// v0.0.567: Settings Notifications - Type Definitions
// Core types for notification system

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub enum NotificationPriority {
    /// Low priority (informational)
    Low,
    /// Normal priority
    #[default]
    Normal,
    /// High priority (important)
    High,
    /// Critical (requires attention)
    Critical,
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    /// Setting changed
    SettingChanged,
    /// Profile switched
    ProfileSwitched,
    /// Sync completed
    SyncCompleted,
    /// Sync conflict
    SyncConflict,
    /// Validation warning
    ValidationWarning,
    /// Import completed
    ImportCompleted,
    /// Export completed
    ExportCompleted,
    /// Backup created
    BackupCreated,
    /// Setting reset
    SettingReset,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SettingChanged => write!(f, "Setting Changed"),
            Self::ProfileSwitched => write!(f, "Profile Switched"),
            Self::SyncCompleted => write!(f, "Sync Completed"),
            Self::SyncConflict => write!(f, "Sync Conflict"),
            Self::ValidationWarning => write!(f, "Validation Warning"),
            Self::ImportCompleted => write!(f, "Import Completed"),
            Self::ExportCompleted => write!(f, "Export Completed"),
            Self::BackupCreated => write!(f, "Backup Created"),
            Self::SettingReset => write!(f, "Setting Reset"),
        }
    }
}

/// A single notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsNotification {
    /// Notification ID
    pub id: u64,
    /// Notification type
    pub notification_type: NotificationType,
    /// Priority level
    pub priority: NotificationPriority,
    /// Title/summary
    pub title: String,
    /// Detailed message
    pub message: String,
    /// Related category (if any)
    pub category: Option<SettingsCategory>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Has been read
    pub read: bool,
    /// Has been dismissed
    pub dismissed: bool,
}

impl SettingsNotification {
    /// Create new notification
    pub fn new(
        id: u64,
        notification_type: NotificationType,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id,
            notification_type,
            priority: NotificationPriority::Normal,
            title: title.into(),
            message: message.into(),
            category: None,
            timestamp: chrono::Utc::now(),
            read: false,
            dismissed: false,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Mark as read
    pub fn mark_read(&mut self) {
        self.read = true;
    }

    /// Dismiss notification
    pub fn dismiss(&mut self) {
        self.dismissed = true;
    }

    /// Age of notification
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Enable notifications
    pub enabled: bool,
    /// Minimum priority to show
    pub min_priority: NotificationPriority,
    /// Show setting changes
    pub show_changes: bool,
    /// Show sync events
    pub show_sync: bool,
    /// Show validation warnings
    pub show_validation: bool,
    /// Auto-dismiss after seconds (0 = never)
    pub auto_dismiss_secs: u64,
    /// Max notifications to keep
    pub max_history: usize,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            min_priority: NotificationPriority::Low,
            show_changes: true,
            show_sync: true,
            show_validation: true,
            auto_dismiss_secs: 0,
            max_history: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", NotificationPriority::High), "High");
        assert_eq!(format!("{}", NotificationPriority::Critical), "Critical");
    }

    #[test]
    fn test_notification_type_display() {
        assert_eq!(format!("{}", NotificationType::SettingChanged), "Setting Changed");
        assert_eq!(format!("{}", NotificationType::SyncCompleted), "Sync Completed");
    }

    #[test]
    fn test_notification_new() {
        let n = SettingsNotification::new(1, NotificationType::SettingChanged, "Test", "Message");
        assert_eq!(n.id, 1);
        assert_eq!(n.title, "Test");
        assert!(!n.read);
    }

    #[test]
    fn test_notification_with_priority() {
        let n = SettingsNotification::new(1, NotificationType::SyncConflict, "Test", "Msg")
            .with_priority(NotificationPriority::Critical);
        assert_eq!(n.priority, NotificationPriority::Critical);
    }

    #[test]
    fn test_notification_mark_read() {
        let mut n = SettingsNotification::new(1, NotificationType::BackupCreated, "Test", "Msg");
        assert!(!n.read);
        n.mark_read();
        assert!(n.read);
    }
}
