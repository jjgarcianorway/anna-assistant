// v0.0.533: Notification Record (Phase 109)
// Individual notification record with lifecycle management

use serde::{Deserialize, Serialize};
use super::types::{NotificationChannel, NotificationPriority, DeliveryStatus};

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
