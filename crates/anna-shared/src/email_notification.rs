//! Email Notification System - Phase 84
//!
//! Tracks email configuration and notification history for long-running tasks.
//! VISION.md: "Ask for email (store for future)" and "Send email with chain of thoughts"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Email notification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Cancelled,
    Queued,
}

impl NotificationStatus {
    pub fn name(&self) -> &'static str {
        match self {
            NotificationStatus::Pending => "Pending",
            NotificationStatus::Sent => "Sent",
            NotificationStatus::Failed => "Failed",
            NotificationStatus::Cancelled => "Cancelled",
            NotificationStatus::Queued => "Queued",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            NotificationStatus::Pending => "...",
            NotificationStatus::Sent => "✓",
            NotificationStatus::Failed => "✗",
            NotificationStatus::Cancelled => "-",
            NotificationStatus::Queued => "~",
        }
    }
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationType {
    TaskComplete,
    TaskFailed,
    ResearchResult,
    TicketUpdate,
    SystemAlert,
    ScheduledReport,
    Custom,
}

impl NotificationType {
    pub fn name(&self) -> &'static str {
        match self {
            NotificationType::TaskComplete => "Task Complete",
            NotificationType::TaskFailed => "Task Failed",
            NotificationType::ResearchResult => "Research Result",
            NotificationType::TicketUpdate => "Ticket Update",
            NotificationType::SystemAlert => "System Alert",
            NotificationType::ScheduledReport => "Scheduled Report",
            NotificationType::Custom => "Custom",
        }
    }
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// User's email address
    pub address: String,
    /// Whether user consented to notifications
    pub consent_given: bool,
    /// When consent was given
    pub consent_timestamp: Option<u64>,
    /// Preferred notification types
    pub enabled_types: Vec<NotificationType>,
    /// Do not disturb hours (start, end in 24h format)
    pub dnd_hours: Option<(u8, u8)>,
    /// Maximum emails per day
    pub daily_limit: Option<u32>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            address: String::new(),
            consent_given: false,
            consent_timestamp: None,
            enabled_types: vec![
                NotificationType::TaskComplete,
                NotificationType::TaskFailed,
                NotificationType::ResearchResult,
            ],
            dnd_hours: None,
            daily_limit: Some(10),
        }
    }
}

/// A notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// Unique notification ID
    pub id: String,
    /// Notification type
    pub notification_type: NotificationType,
    /// Status
    pub status: NotificationStatus,
    /// Subject line
    pub subject: String,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// When created
    pub created_at: u64,
    /// When sent (if sent)
    pub sent_at: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
    /// Retry count
    pub retry_count: u32,
}

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

/// Format email tracker for display
pub fn format_email_tracker(tracker: &EmailNotificationTracker) -> String {
    let mut lines = vec!["=== Email Notifications ===".to_string()];
    lines.push(String::new());

    // Config status
    if let Some(config) = &tracker.config {
        if config.consent_given {
            lines.push(format!("Email: {} (consent given)", config.address));
        } else {
            lines.push("Email: Not configured (no consent)".to_string());
        }
    } else {
        lines.push("Email: Not configured".to_string());
    }

    if tracker.notifications.is_empty() {
        lines.push("No notifications yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(String::new());
    lines.push(format!("Total notifications: {}", tracker.total_count()));
    lines.push(format!("Sent: {}", tracker.sent_count()));
    lines.push(format!("Success rate: {:.1}%", tracker.success_rate()));
    lines.push(format!("Sent today: {}", tracker.sent_today));

    // Daily limit
    if tracker.daily_limit_reached() {
        lines.push("Daily limit reached!".to_string());
    }

    // Recent
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent:".to_string());
        for n in recent {
            lines.push(format!(
                "  [{}] {} - {}",
                n.status.symbol(),
                n.notification_type.name(),
                n.subject
            ));
        }
    }

    lines.join("\n")
}

/// Format email tracker compact
pub fn format_email_tracker_compact(tracker: &EmailNotificationTracker) -> String {
    let configured = if tracker.is_configured() { "configured" } else { "not configured" };
    format!(
        "Email: {} | {} sent | {:.0}% success",
        configured,
        tracker.sent_count(),
        tracker.success_rate()
    )
}

/// Format email tracker one-line
pub fn format_email_tracker_oneline(tracker: &EmailNotificationTracker) -> String {
    format!(
        "{} notifications ({} sent)",
        tracker.total_count(),
        tracker.sent_count()
    )
}

/// Check if query is about email notifications
pub fn is_email_notification_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "email notification",
        "email me",
        "notify me",
        "send email",
        "notification history",
        "email config",
        "configure email",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about email notifications
pub fn email_fun_fact(tracker: &EmailNotificationTracker) -> String {
    if tracker.notifications.is_empty() {
        return "No email notifications sent yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has sent {} email notifications.",
            tracker.sent_count()
        ),
        format!(
            "Email success rate is {:.1}%.",
            tracker.success_rate()
        ),
        format!(
            "{} notifications sent today.",
            tracker.sent_today
        ),
        format!(
            "{} notifications are pending.",
            tracker.pending().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(ntype: NotificationType, status: NotificationStatus) -> NotificationRecord {
        NotificationRecord {
            id: format!("NOT-{:?}", ntype),
            notification_type: ntype,
            status,
            subject: "Test notification".to_string(),
            ticket_id: None,
            created_at: 1234567890,
            sent_at: None,
            error: None,
            retry_count: 0,
        }
    }

    #[test]
    fn test_notification_status() {
        assert_eq!(NotificationStatus::Sent.name(), "Sent");
        assert_eq!(NotificationStatus::Failed.symbol(), "✗");
    }

    #[test]
    fn test_notification_type() {
        assert_eq!(NotificationType::TaskComplete.name(), "Task Complete");
        assert_eq!(NotificationType::ResearchResult.name(), "Research Result");
    }

    #[test]
    fn test_configure_email() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), true, 1234567890);

        assert!(tracker.is_configured());
        assert_eq!(tracker.config.as_ref().unwrap().address, "user@example.com");
    }

    #[test]
    fn test_not_configured_without_consent() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), false, 1234567890);

        assert!(!tracker.is_configured());
    }

    #[test]
    fn test_record_notification() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.sent_count(), 1);
        assert_eq!(tracker.sent_today, 1);
    }

    #[test]
    fn test_update_status() {
        let mut tracker = EmailNotificationTracker::new();
        let mut notification = make_notification(NotificationType::TaskComplete, NotificationStatus::Pending);
        notification.id = "NOT-001".to_string();
        tracker.record(notification);

        assert!(tracker.update_status("NOT-001", NotificationStatus::Sent, None));
        assert_eq!(tracker.sent_count(), 1);
    }

    #[test]
    fn test_daily_limit() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), true, 1234567890);
        tracker.config.as_mut().unwrap().daily_limit = Some(2);

        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));
        assert!(!tracker.daily_limit_reached());

        tracker.record(make_notification(NotificationType::TaskFailed, NotificationStatus::Sent));
        assert!(tracker.daily_limit_reached());
    }

    #[test]
    fn test_reset_daily() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.sent_today = 5;
        tracker.reset_daily("2025-12-13");

        assert_eq!(tracker.sent_today, 0);
        assert_eq!(tracker.last_reset_date, Some("2025-12-13".to_string()));
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Failed));

        assert!((tracker.success_rate() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_format_email_tracker() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.configure("user@example.com".to_string(), true, 1234567890);
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));

        let output = format_email_tracker(&tracker);
        assert!(output.contains("Email Notifications"));
        assert!(output.contains("user@example.com"));
    }

    #[test]
    fn test_is_email_notification_query() {
        assert!(is_email_notification_query("email me when done"));
        assert!(is_email_notification_query("configure email"));
        assert!(is_email_notification_query("notification history"));
        assert!(!is_email_notification_query("what is the weather?"));
    }

    #[test]
    fn test_email_fun_fact() {
        let mut tracker = EmailNotificationTracker::new();
        tracker.record(make_notification(NotificationType::TaskComplete, NotificationStatus::Sent));

        let fact = email_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
