// v0.0.567: Settings Notifications (Phase 143)
// Notify users about settings changes and important events

mod types;
mod manager;
mod helpers;

// Re-export all public types and functions
pub use types::{
    NotificationPriority,
    NotificationType,
    SettingsNotification,
    NotificationPreferences,
};

pub use manager::NotificationManager;

pub use helpers::{
    format_notifications,
    is_notification_query,
    notifications_fun_fact,
};
