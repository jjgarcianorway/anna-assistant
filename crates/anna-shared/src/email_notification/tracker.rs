//! Email notification tracker implementation

use super::types::{EmailConfig, NotificationRecord, NotificationStatus, NotificationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Email notification tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailNotificationTracker {
    /// Email configuration
    pub config: Option<EmailConfig>,
    /// Notification history
    pub notifications: Vec<NotificationRecord>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Emails sent today
    pub sent_today: u32,
    /// Last reset date (for daily limit)
    pub last_reset_date: Option<String>,
}

impl EmailNotificationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure email
    pub fn configure(&mut self, address: String, consent: bool, timestamp: u64) {
        self.config = Some(EmailConfig {
            address,
            consent_given: consent,
            consent_timestamp: Some(timestamp),
            ..EmailConfig::default()
        });
    }

    /// Check if email is configured and consented
    pub fn is_configured(&self) -> bool {
        self.config.as_ref().map(|c| c.consent_given && !c.address.is_empty()).unwrap_or(false)
    }

    /// Record a notification
    pub fn record(&mut self, notification: NotificationRecord) {
        *self.by_status.entry(notification.status.name().to_string()).or_insert(0) += 1;
        *self.by_type.entry(notification.notification_type.name().to_string()).or_insert(0) += 1;

        if notification.status == NotificationStatus::Sent {
            self.sent_today += 1;
        }

        self.notifications.push(notification);
    }

    /// Update notification status
    pub fn update_status(&mut self, id: &str, status: NotificationStatus, error: Option<String>) -> bool {
        let found = self.notifications.iter().position(|n| n.id == id);
        if let Some(idx) = found {
            let old_status = self.notifications[idx].status;
            self.notifications[idx].status = status;
            self.notifications[idx].error = error;

            // Update counts
            if let Some(count) = self.by_status.get_mut(old_status.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_status.entry(status.name().to_string()).or_insert(0) += 1;

            if status == NotificationStatus::Sent {
                self.notifications[idx].sent_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                );
                self.sent_today += 1;
            }

            true
        } else {
            false
        }
    }

    /// Check if daily limit reached
    pub fn daily_limit_reached(&self) -> bool {
        self.config
            .as_ref()
            .and_then(|c| c.daily_limit)
            .map(|limit| self.sent_today >= limit)
            .unwrap_or(false)
    }

    /// Reset daily counter (call at midnight)
    pub fn reset_daily(&mut self, date: &str) {
        self.sent_today = 0;
        self.last_reset_date = Some(date.to_string());
    }

    /// Get pending notifications
    pub fn pending(&self) -> Vec<&NotificationRecord> {
        self.notifications.iter().filter(|n| n.status == NotificationStatus::Pending).collect()
    }

    /// Get sent notifications
    pub fn sent(&self) -> Vec<&NotificationRecord> {
        self.notifications.iter().filter(|n| n.status == NotificationStatus::Sent).collect()
    }

    /// Get failed notifications
    pub fn failed(&self) -> Vec<&NotificationRecord> {
        self.notifications.iter().filter(|n| n.status == NotificationStatus::Failed).collect()
    }

    /// Get recent notifications
    pub fn recent(&self, limit: usize) -> Vec<&NotificationRecord> {
        self.notifications.iter().rev().take(limit).collect()
    }

    /// Get notifications by type
    pub fn by_notification_type(&self, ntype: NotificationType) -> Vec<&NotificationRecord> {
        self.notifications.iter().filter(|n| n.notification_type == ntype).collect()
    }

    /// Total notification count
    pub fn total_count(&self) -> usize {
        self.notifications.len()
    }

    /// Sent count
    pub fn sent_count(&self) -> usize {
        self.notifications.iter().filter(|n| n.status == NotificationStatus::Sent).count()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let sent = self.sent_count();
        let failed = self.failed().len();
        let total = sent + failed;
        if total == 0 {
            100.0
        } else {
            (sent as f64 / total as f64) * 100.0
        }
    }
}
