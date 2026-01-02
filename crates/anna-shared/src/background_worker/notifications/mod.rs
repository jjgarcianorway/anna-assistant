//! Notification system (v0.0.430).
//!
//! Supports multiple notification channels:
//! - Email (via sendmail/msmtp)
//! - Desktop notifications (notify-send)
//! - Wall messages (terminal broadcast)
//!
//! Rules:
//! - No notification without explicit user config
//! - Rate-limited to prevent spam
//! - All channels visible in status

mod dispatcher;
mod result;
mod types;
mod utils;

pub use dispatcher::NotificationDispatcher;
pub use result::{NotificationStatus, NotifyResult};
pub use types::{AlertPriority, EmailConfig, NotificationChannel, NotificationConfig};
