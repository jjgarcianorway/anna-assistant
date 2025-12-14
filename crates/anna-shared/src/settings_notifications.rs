// v0.0.567: Settings Notifications (Phase 143)
// Notify users about settings changes and important events

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

/// Notification manager
#[derive(Debug, Clone, Default)]
pub struct NotificationManager {
    /// All notifications
    notifications: Vec<SettingsNotification>,
    /// Next notification ID
    next_id: u64,
    /// Preferences
    pub preferences: NotificationPreferences,
}

impl NotificationManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a notification
    pub fn notify(&mut self, notification_type: NotificationType, title: &str, message: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = SettingsNotification::new(id, notification_type, title, message);
        self.notifications.push(notification);

        // Trim history
        if self.notifications.len() > self.preferences.max_history {
            self.notifications.remove(0);
        }

        id
    }

    /// Add notification with priority
    pub fn notify_with_priority(
        &mut self,
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: &str,
        message: &str,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = SettingsNotification::new(id, notification_type, title, message)
            .with_priority(priority);
        self.notifications.push(notification);

        if self.notifications.len() > self.preferences.max_history {
            self.notifications.remove(0);
        }

        id
    }

    /// Get notification by ID
    pub fn get(&self, id: u64) -> Option<&SettingsNotification> {
        self.notifications.iter().find(|n| n.id == id)
    }

    /// Get all unread notifications
    pub fn unread(&self) -> Vec<&SettingsNotification> {
        self.notifications.iter().filter(|n| !n.read && !n.dismissed).collect()
    }

    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.read && !n.dismissed).count()
    }

    /// Get all notifications
    pub fn all(&self) -> &[SettingsNotification] {
        &self.notifications
    }

    /// Get recent notifications
    pub fn recent(&self, count: usize) -> Vec<&SettingsNotification> {
        self.notifications.iter().rev().take(count).collect()
    }

    /// Mark notification as read
    pub fn mark_read(&mut self, id: u64) -> bool {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.mark_read();
            return true;
        }
        false
    }

    /// Mark all as read
    pub fn mark_all_read(&mut self) {
        for n in &mut self.notifications {
            n.mark_read();
        }
    }

    /// Dismiss notification
    pub fn dismiss(&mut self, id: u64) -> bool {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.dismiss();
            return true;
        }
        false
    }

    /// Dismiss all
    pub fn dismiss_all(&mut self) {
        for n in &mut self.notifications {
            n.dismiss();
        }
    }

    /// Clear all notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// Get by type
    pub fn by_type(&self, notification_type: NotificationType) -> Vec<&SettingsNotification> {
        self.notifications
            .iter()
            .filter(|n| n.notification_type == notification_type)
            .collect()
    }

    /// Get by priority (at least)
    pub fn by_priority(&self, min_priority: NotificationPriority) -> Vec<&SettingsNotification> {
        self.notifications
            .iter()
            .filter(|n| n.priority >= min_priority)
            .collect()
    }

    /// Notify setting changed
    pub fn setting_changed(&mut self, category: SettingsCategory, field: &str, old: &str, new: &str) -> u64 {
        let id = self.notify(
            NotificationType::SettingChanged,
            &format!("{} changed", field),
            &format!("{}: {} → {}", category, old, new),
        );
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.category = Some(category);
        }
        id
    }

    /// Notify profile switched
    pub fn profile_switched(&mut self, from: &str, to: &str) -> u64 {
        self.notify(
            NotificationType::ProfileSwitched,
            "Profile switched",
            &format!("Switched from '{}' to '{}'", from, to),
        )
    }

    /// Notify sync completed
    pub fn sync_completed(&mut self, direction: &str) -> u64 {
        self.notify(
            NotificationType::SyncCompleted,
            "Sync completed",
            &format!("Settings {} successfully", direction),
        )
    }
}

/// Format notifications for display
pub fn format_notifications(manager: &NotificationManager) -> String {
    let mut output = String::new();

    output.push_str("=== Notifications ===\n\n");

    let unread = manager.unread();
    if unread.is_empty() {
        output.push_str("No new notifications.\n");
        return output;
    }

    output.push_str(&format!("{} unread notification(s)\n\n", unread.len()));

    for n in unread {
        let priority_marker = match n.priority {
            NotificationPriority::Critical => "!!!",
            NotificationPriority::High => "!!",
            NotificationPriority::Normal => "",
            NotificationPriority::Low => "",
        };
        output.push_str(&format!(
            "• [{}] {}{}\n  {}\n  {}\n\n",
            n.notification_type,
            n.title,
            priority_marker,
            n.message,
            n.timestamp.format("%Y-%m-%d %H:%M")
        ));
    }

    output
}

/// Check if query is about notifications
pub fn is_notification_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("notification")
        || lower.contains("alerts")
        || lower.contains("unread")
        || lower.contains("messages")
}

/// Fun fact about notifications
pub fn notifications_fun_fact() -> &'static str {
    "Anna can notify you about important settings changes and sync events!"
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

    #[test]
    fn test_manager_new() {
        let manager = NotificationManager::new();
        assert_eq!(manager.unread_count(), 0);
    }

    #[test]
    fn test_manager_notify() {
        let mut manager = NotificationManager::new();
        let id = manager.notify(NotificationType::SettingChanged, "Test", "Message");
        assert_eq!(id, 0);
        assert_eq!(manager.unread_count(), 1);
    }

    #[test]
    fn test_manager_mark_read() {
        let mut manager = NotificationManager::new();
        let id = manager.notify(NotificationType::SettingChanged, "Test", "Msg");
        assert_eq!(manager.unread_count(), 1);
        manager.mark_read(id);
        assert_eq!(manager.unread_count(), 0);
    }

    #[test]
    fn test_manager_dismiss() {
        let mut manager = NotificationManager::new();
        let id = manager.notify(NotificationType::SyncCompleted, "Test", "Msg");
        assert!(manager.dismiss(id));
        assert_eq!(manager.unread_count(), 0);
    }

    #[test]
    fn test_manager_setting_changed() {
        let mut manager = NotificationManager::new();
        manager.setting_changed(SettingsCategory::Privacy, "telemetry", "on", "off");
        assert_eq!(manager.unread_count(), 1);
    }

    #[test]
    fn test_format_notifications() {
        let mut manager = NotificationManager::new();
        manager.notify(NotificationType::BackupCreated, "Backup", "Created");
        let output = format_notifications(&manager);
        assert!(output.contains("Notifications"));
    }

    #[test]
    fn test_is_notification_query() {
        assert!(is_notification_query("show notifications"));
        assert!(is_notification_query("any unread alerts"));
        assert!(!is_notification_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notifications_fun_fact();
        assert!(fact.contains("notify"));
    }
}
