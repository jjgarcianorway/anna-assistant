// v0.0.533: Notification Tracker (Phase 109)
// Tracks user notifications via email, libnotify, wall per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notification channel type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Libnotify,
    Wall,
    Terminal,
    Log,
}

impl Default for NotificationChannel {
    fn default() -> Self {
        Self::Terminal
    }
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "Email"),
            Self::Libnotify => write!(f, "Desktop Notification"),
            Self::Wall => write!(f, "Wall Message"),
            Self::Terminal => write!(f, "Terminal"),
            Self::Log => write!(f, "Log"),
        }
    }
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Urgent => write!(f, "Urgent"),
        }
    }
}

/// Notification delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DeliveryStatus {
    #[default]
    Pending,
    Sent,
    Delivered,
    Failed,
    Suppressed,
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Sent => write!(f, "Sent"),
            Self::Delivered => write!(f, "Delivered"),
            Self::Failed => write!(f, "Failed"),
            Self::Suppressed => write!(f, "Suppressed"),
        }
    }
}

/// Individual notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: String,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub subject: String,
    pub message: String,
    pub ticket_id: Option<String>,
    pub status: DeliveryStatus,
    pub created_at: String,
    pub sent_at: Option<String>,
    pub error: Option<String>,
}

impl NotificationRecord {
    /// Create new notification
    pub fn new(
        id: &str,
        channel: NotificationChannel,
        priority: NotificationPriority,
        subject: &str,
        message: &str,
        timestamp: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            channel,
            priority,
            subject: subject.to_string(),
            message: message.to_string(),
            ticket_id: None,
            status: DeliveryStatus::Pending,
            created_at: timestamp.to_string(),
            sent_at: None,
            error: None,
        }
    }

    /// Link to ticket
    pub fn link_ticket(&mut self, ticket_id: &str) {
        self.ticket_id = Some(ticket_id.to_string());
    }

    /// Mark as sent
    pub fn mark_sent(&mut self, timestamp: &str) {
        self.status = DeliveryStatus::Sent;
        self.sent_at = Some(timestamp.to_string());
    }

    /// Mark as delivered
    pub fn mark_delivered(&mut self) {
        self.status = DeliveryStatus::Delivered;
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: &str) {
        self.status = DeliveryStatus::Failed;
        self.error = Some(error.to_string());
    }

    /// Suppress notification
    pub fn suppress(&mut self) {
        self.status = DeliveryStatus::Suppressed;
    }

    /// Is pending?
    pub fn is_pending(&self) -> bool {
        self.status == DeliveryStatus::Pending
    }
}

/// Notification tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationTracker {
    notifications: Vec<NotificationRecord>,
    next_id: u32,
    email_address: Option<String>,
    spam_cooldown_ms: u64,
    last_notification_ms: HashMap<NotificationChannel, u64>,
}

impl NotificationTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            next_id: 1,
            email_address: None,
            spam_cooldown_ms: 60000, // 1 minute default
            last_notification_ms: HashMap::new(),
        }
    }

    /// Set email address
    pub fn set_email(&mut self, email: &str) {
        self.email_address = Some(email.to_string());
    }

    /// Set spam cooldown
    pub fn set_cooldown(&mut self, ms: u64) {
        self.spam_cooldown_ms = ms;
    }

    /// Create notification
    pub fn create(
        &mut self,
        channel: NotificationChannel,
        priority: NotificationPriority,
        subject: &str,
        message: &str,
        timestamp: &str,
    ) -> String {
        let id = format!("NOTIF-{:05}", self.next_id);
        self.next_id += 1;

        let notif = NotificationRecord::new(&id, channel, priority, subject, message, timestamp);
        self.notifications.push(notif);
        id
    }

    /// Get notification by ID
    pub fn get(&self, id: &str) -> Option<&NotificationRecord> {
        self.notifications.iter().find(|n| n.id == id)
    }

    /// Get mutable notification
    pub fn get_mut(&mut self, id: &str) -> Option<&mut NotificationRecord> {
        self.notifications.iter_mut().find(|n| n.id == id)
    }

    /// Check if should suppress (anti-spam)
    pub fn should_suppress(&self, channel: &NotificationChannel, current_ms: u64) -> bool {
        if let Some(last) = self.last_notification_ms.get(channel) {
            return current_ms - last < self.spam_cooldown_ms;
        }
        false
    }

    /// Record notification sent
    pub fn record_sent(&mut self, channel: NotificationChannel, timestamp_ms: u64) {
        self.last_notification_ms.insert(channel, timestamp_ms);
    }

    /// Get pending notifications
    pub fn pending(&self) -> Vec<&NotificationRecord> {
        self.notifications.iter().filter(|n| n.is_pending()).collect()
    }

    /// Get notifications by channel
    pub fn by_channel(&self, channel: &NotificationChannel) -> Vec<&NotificationRecord> {
        self.notifications
            .iter()
            .filter(|n| &n.channel == channel)
            .collect()
    }

    /// Get notifications for ticket
    pub fn for_ticket(&self, ticket_id: &str) -> Vec<&NotificationRecord> {
        self.notifications
            .iter()
            .filter(|n| n.ticket_id.as_deref() == Some(ticket_id))
            .collect()
    }

    /// Channel stats
    pub fn channel_stats(&self) -> HashMap<NotificationChannel, usize> {
        let mut stats = HashMap::new();
        for n in &self.notifications {
            *stats.entry(n.channel.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// Delivery stats
    pub fn delivery_stats(&self) -> HashMap<DeliveryStatus, usize> {
        let mut stats = HashMap::new();
        for n in &self.notifications {
            *stats.entry(n.status).or_insert(0) += 1;
        }
        stats
    }

    /// Total notifications
    pub fn total(&self) -> usize {
        self.notifications.len()
    }

    /// Has email configured?
    pub fn has_email(&self) -> bool {
        self.email_address.is_some()
    }

    /// Get recent notifications
    pub fn recent(&self, n: usize) -> Vec<&NotificationRecord> {
        self.notifications.iter().rev().take(n).collect()
    }
}

/// Format notification for display
pub fn format_notification(notif: &NotificationRecord) -> String {
    format!(
        "{} [{}]\n  Channel: {} | Priority: {}\n  Subject: {}\n  Status: {}{}",
        notif.id,
        notif.created_at,
        notif.channel,
        notif.priority,
        notif.subject,
        notif.status,
        if let Some(err) = &notif.error {
            format!("\n  Error: {}", err)
        } else {
            String::new()
        }
    )
}

/// Format notification compact
pub fn format_notification_compact(notif: &NotificationRecord) -> String {
    format!(
        "{}: {} [{}] - {}",
        notif.id, notif.subject, notif.channel, notif.status
    )
}

/// Format notification oneline
pub fn format_notification_oneline(notif: &NotificationRecord) -> String {
    format!("{} [{}]", notif.id, notif.status)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &NotificationTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Notification Tracker ===\n\n");

    output.push_str(&format!("Total: {}\n", tracker.total()));
    output.push_str(&format!("Pending: {}\n", tracker.pending().len()));
    output.push_str(&format!(
        "Email: {}\n\n",
        if tracker.has_email() { "Configured" } else { "Not set" }
    ));

    output.push_str("--- By Channel ---\n");
    for (channel, count) in tracker.channel_stats() {
        output.push_str(&format!("  {}: {}\n", channel, count));
    }

    output.push_str("\n--- Delivery Status ---\n");
    for (status, count) in tracker.delivery_stats() {
        output.push_str(&format!("  {}: {}\n", status, count));
    }

    output
}

/// Check if query is notification-related
pub fn is_notification_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("notif")
        || lower.contains("email")
        || lower.contains("alert")
        || lower.contains("message")
        || lower.contains("remind")
}

/// Fun fact about notifications
pub fn notification_fun_fact() -> &'static str {
    "Anna respects your focus - she uses smart anti-spam to avoid flooding you with notifications. Quality over quantity!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = NotificationRecord::new(
            "NOTIF-001",
            NotificationChannel::Email,
            NotificationPriority::High,
            "Ticket Updated",
            "Your ticket has been resolved",
            "2024-01-01",
        );
        assert_eq!(notif.subject, "Ticket Updated");
        assert!(notif.is_pending());
    }

    #[test]
    fn test_notification_lifecycle() {
        let mut notif = NotificationRecord::new(
            "N-001",
            NotificationChannel::Libnotify,
            NotificationPriority::Normal,
            "Test",
            "Message",
            "ts",
        );
        notif.mark_sent("ts2");
        assert_eq!(notif.status, DeliveryStatus::Sent);
        notif.mark_delivered();
        assert_eq!(notif.status, DeliveryStatus::Delivered);
    }

    #[test]
    fn test_notification_failed() {
        let mut notif = NotificationRecord::new(
            "N-001",
            NotificationChannel::Email,
            NotificationPriority::Normal,
            "Test",
            "Msg",
            "ts",
        );
        notif.mark_failed("Connection refused");
        assert_eq!(notif.status, DeliveryStatus::Failed);
        assert!(notif.error.is_some());
    }

    #[test]
    fn test_tracker_create() {
        let mut tracker = NotificationTracker::new();
        let id = tracker.create(
            NotificationChannel::Terminal,
            NotificationPriority::Low,
            "Test",
            "Msg",
            "ts",
        );
        assert_eq!(tracker.total(), 1);
        assert!(tracker.get(&id).is_some());
    }

    #[test]
    fn test_set_email() {
        let mut tracker = NotificationTracker::new();
        assert!(!tracker.has_email());
        tracker.set_email("user@example.com");
        assert!(tracker.has_email());
    }

    #[test]
    fn test_should_suppress() {
        let mut tracker = NotificationTracker::new();
        tracker.set_cooldown(1000);
        tracker.record_sent(NotificationChannel::Libnotify, 100);
        assert!(tracker.should_suppress(&NotificationChannel::Libnotify, 500));
        assert!(!tracker.should_suppress(&NotificationChannel::Libnotify, 1200));
    }

    #[test]
    fn test_by_channel() {
        let mut tracker = NotificationTracker::new();
        tracker.create(NotificationChannel::Email, NotificationPriority::Normal, "A", "m", "ts");
        tracker.create(NotificationChannel::Email, NotificationPriority::Normal, "B", "m", "ts");
        tracker.create(NotificationChannel::Wall, NotificationPriority::Normal, "C", "m", "ts");
        assert_eq!(tracker.by_channel(&NotificationChannel::Email).len(), 2);
    }

    #[test]
    fn test_channel_stats() {
        let mut tracker = NotificationTracker::new();
        tracker.create(NotificationChannel::Terminal, NotificationPriority::Normal, "A", "m", "ts");
        tracker.create(NotificationChannel::Log, NotificationPriority::Normal, "B", "m", "ts");
        let stats = tracker.channel_stats();
        assert_eq!(stats.get(&NotificationChannel::Terminal), Some(&1));
    }

    #[test]
    fn test_is_notification_query() {
        assert!(is_notification_query("Send me an email"));
        assert!(is_notification_query("Show notifications"));
        assert!(is_notification_query("Set up alerts"));
        assert!(!is_notification_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = notification_fun_fact();
        assert!(fact.contains("spam") || fact.contains("focus"));
    }
}
