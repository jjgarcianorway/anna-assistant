// v0.0.567: Settings Notifications - Helper Functions
// Formatting and utility functions for notifications

use super::manager::NotificationManager;
use super::types::NotificationPriority;

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
    use crate::settings_notifications::types::NotificationType;

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
