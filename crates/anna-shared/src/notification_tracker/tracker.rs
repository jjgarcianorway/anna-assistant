// v0.0.533: Notification Tracker (Phase 109)
// Main tracker for managing notifications with anti-spam

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{NotificationChannel, NotificationPriority, DeliveryStatus};
use super::record::NotificationRecord;

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
