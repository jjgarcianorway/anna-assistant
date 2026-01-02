//! Basic types for specialist conversations.

use serde::{Deserialize, Serialize};

/// Speaker in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Speaker {
    Anna,
    Junior(String),
    Senior(String),
    User,
}

impl Speaker {
    pub fn name(&self) -> &str {
        match self {
            Speaker::Anna => "Anna",
            Speaker::Junior(name) => name,
            Speaker::Senior(name) => name,
            Speaker::User => "User",
        }
    }

    pub fn role(&self) -> &str {
        match self {
            Speaker::Anna => "Assistant",
            Speaker::Junior(_) => "Junior",
            Speaker::Senior(_) => "Senior",
            Speaker::User => "User",
        }
    }

    pub fn is_specialist(&self) -> bool {
        matches!(self, Speaker::Junior(_) | Speaker::Senior(_))
    }
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Query,
    Response,
    Clarification,
    Escalation,
    Resolution,
    Thanks,
    Confirmation,
}

impl MessageType {
    pub fn description(&self) -> &'static str {
        match self {
            MessageType::Query => "asked",
            MessageType::Response => "replied",
            MessageType::Clarification => "clarified",
            MessageType::Escalation => "escalated",
            MessageType::Resolution => "resolved",
            MessageType::Thanks => "thanked",
            MessageType::Confirmation => "confirmed",
        }
    }
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Who sent this message
    pub from: Speaker,
    /// Who received this message
    pub to: Speaker,
    /// Message content
    pub content: String,
    /// Type of message
    pub message_type: MessageType,
    /// Timestamp (Unix timestamp)
    pub timestamp: u64,
    /// Reliability if this is a response
    pub reliability: Option<u8>,
    /// Risk level if applicable
    pub risk_level: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker() {
        assert_eq!(Speaker::Anna.name(), "Anna");
        assert_eq!(Speaker::Junior("Bob".to_string()).role(), "Junior");
        assert!(Speaker::Senior("Alice".to_string()).is_specialist());
        assert!(!Speaker::Anna.is_specialist());
    }

    #[test]
    fn test_message_type() {
        assert_eq!(MessageType::Query.description(), "asked");
        assert_eq!(MessageType::Resolution.description(), "resolved");
    }
}
