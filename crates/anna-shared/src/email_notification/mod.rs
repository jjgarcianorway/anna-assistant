//! Email Notification System - Phase 84
//!
//! Tracks email configuration and notification history for long-running tasks.
//! VISION.md: "Ask for email (store for future)" and "Send email with chain of thoughts"

mod formatting;
mod tracker;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use formatting::{
    format_email_tracker, format_email_tracker_compact, format_email_tracker_oneline,
};
pub use tracker::EmailNotificationTracker;
pub use types::{EmailConfig, NotificationRecord, NotificationStatus, NotificationType};
pub use utils::{email_fun_fact, is_email_notification_query};
