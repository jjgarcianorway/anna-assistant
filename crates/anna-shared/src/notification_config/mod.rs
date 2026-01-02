//! Notification Configuration via Natural Language (v0.0.470).
//!
//! Allows users to configure notification settings through natural language:
//! - "set my email to user@example.com"
//! - "enable desktop notifications"
//! - "disable wall messages"
//! - "set quiet hours 22:00 to 08:00"
//!
//! Per VISION.md: "All settings changeable through annactl in natural language"

mod types;
mod parsers;
mod detector;
mod apply;

// Re-export public API
pub use types::NotifyConfigChange;
pub use detector::{detect_notify_config, is_show_notifications};
pub use apply::{apply_notify_change, format_notification_settings};
