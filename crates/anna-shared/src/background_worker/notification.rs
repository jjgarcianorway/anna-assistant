//! Notification types for background jobs (v0.0.430).

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A pending notification to be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingNotification {
    /// Unique ID
    pub id: String,
    /// Subject/title
    pub subject: String,
    /// Message body
    pub body: String,
    /// Priority (affects channel selection)
    pub priority: NotificationPriority,
    /// Source job ID
    pub source_job_id: Option<String>,
    /// When created
    pub created_at: u64,
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    /// Informational (shown next annactl open)
    Low,
    /// Normal (desktop notification if enabled)
    Normal,
    /// Critical (email + desktop if configured)
    High,
}

impl PendingNotification {
    /// Create a new notification
    pub fn new(subject: &str, body: &str, priority: NotificationPriority) -> Self {
        Self {
            id: generate_notification_id(),
            subject: subject.to_string(),
            body: body.to_string(),
            priority,
            source_job_id: None,
            created_at: now_timestamp(),
        }
    }

    /// Set source job
    pub fn from_job(mut self, job_id: &str) -> Self {
        self.source_job_id = Some(job_id.to_string());
        self
    }
}

/// Generate a unique notification ID
fn generate_notification_id() -> String {
    format!(
        "NOTIF-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
    )
}

/// Get current unix timestamp
fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
