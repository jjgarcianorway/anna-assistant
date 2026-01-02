//! Core interaction types.

use serde::{Deserialize, Serialize};

/// Type of interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    /// Initial dispatch to specialist
    Dispatch,
    /// Response from specialist
    Response,
    /// Escalation to senior
    Escalation,
    /// Clarification request
    Clarification,
    /// Follow-up question
    FollowUp,
    /// Final resolution
    Resolution,
}

impl InteractionType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dispatch => "Dispatch",
            Self::Response => "Response",
            Self::Escalation => "Escalation",
            Self::Clarification => "Clarification",
            Self::FollowUp => "Follow-up",
            Self::Resolution => "Resolution",
        }
    }
}

/// A single interaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    /// Unix timestamp
    pub timestamp: u64,
    /// Source (who initiated)
    pub from: String,
    /// Target (who received)
    pub to: String,
    /// Type of interaction
    pub interaction_type: InteractionType,
    /// Ticket ID if applicable
    pub ticket_id: Option<String>,
    /// Duration in ms if applicable
    pub duration_ms: Option<u64>,
}

impl InteractionRecord {
    /// Create a new interaction record
    pub fn new(from: &str, to: &str, interaction_type: InteractionType, timestamp: u64) -> Self {
        Self {
            timestamp,
            from: from.to_string(),
            to: to.to_string(),
            interaction_type,
            ticket_id: None,
            duration_ms: None,
        }
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: &str) -> Self {
        self.ticket_id = Some(ticket_id.to_string());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_record_new() {
        let record = InteractionRecord::new("Anna", "Desktop Admin", InteractionType::Dispatch, 1000);
        assert_eq!(record.from, "Anna");
        assert_eq!(record.to, "Desktop Admin");
    }

    #[test]
    fn test_interaction_with_ticket() {
        let record = InteractionRecord::new("Anna", "Network", InteractionType::Dispatch, 1000)
            .with_ticket("TKT-001");
        assert_eq!(record.ticket_id, Some("TKT-001".to_string()));
    }

    #[test]
    fn test_interaction_type_display() {
        assert_eq!(InteractionType::Dispatch.display_name(), "Dispatch");
        assert_eq!(InteractionType::Escalation.display_name(), "Escalation");
    }
}
