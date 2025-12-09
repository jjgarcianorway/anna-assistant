//! Ticket message type (v0.0.183).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A message in ticket conversation (v0.0.113)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessage {
    /// Who sent this message ("user", "anna", or staff ID)
    pub sender: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl TicketMessage {
    pub fn from_user(content: String) -> Self {
        Self {
            sender: "user".to_string(),
            content,
            timestamp: Utc::now(),
        }
    }
    pub fn from_anna(content: String) -> Self {
        Self {
            sender: "anna".to_string(),
            content,
            timestamp: Utc::now(),
        }
    }
    pub fn from_staff(staff_id: &str, content: String) -> Self {
        Self {
            sender: staff_id.to_string(),
            content,
            timestamp: Utc::now(),
        }
    }
}
