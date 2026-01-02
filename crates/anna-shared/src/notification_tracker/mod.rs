// v0.0.533: Notification Tracker Module (Phase 109)
// Tracks user notifications via email, libnotify, wall per VISION.md

mod types;
mod record;
mod tracker;
mod formatting;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{NotificationChannel, NotificationPriority, DeliveryStatus};
pub use record::NotificationRecord;
pub use tracker::NotificationTracker;
pub use formatting::{
    format_notification,
    format_notification_compact,
    format_notification_oneline,
    format_tracker_summary,
    is_notification_query,
    notification_fun_fact,
};
