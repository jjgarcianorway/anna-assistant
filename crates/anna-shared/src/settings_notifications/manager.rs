// v0.0.567: Settings Notifications - Manager Implementation
// Manages notification lifecycle and queries

use crate::unified_settings::SettingsCategory;

use super::types::{NotificationPreferences, NotificationPriority, NotificationType, SettingsNotification};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
