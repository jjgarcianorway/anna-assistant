//! Conversation data structure and methods.

use crate::specialist_conversation::types::{ConversationMessage, Speaker};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete conversation thread
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique conversation ID (ticket ID)
    pub id: String,
    /// All messages in order
    pub messages: Vec<ConversationMessage>,
    /// Department/team handling this
    pub department: Option<String>,
    /// Whether conversation is resolved
    pub resolved: bool,
    /// Timestamp when started
    pub started_at: u64,
    /// Timestamp when resolved
    pub resolved_at: Option<u64>,
}

impl Conversation {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            ..Default::default()
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, msg: ConversationMessage) {
        self.messages.push(msg);
    }

    /// Mark as resolved
    pub fn resolve(&mut self) {
        self.resolved = true;
        self.resolved_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
    }

    /// Get all participants
    pub fn participants(&self) -> Vec<&Speaker> {
        let mut seen: HashMap<String, &Speaker> = HashMap::new();
        for msg in &self.messages {
            seen.insert(msg.from.name().to_string(), &msg.from);
            seen.insert(msg.to.name().to_string(), &msg.to);
        }
        seen.into_values().collect()
    }

    /// Count messages by speaker
    pub fn messages_by_speaker(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for msg in &self.messages {
            *counts.entry(msg.from.name().to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Duration in seconds
    pub fn duration_secs(&self) -> u64 {
        match self.resolved_at {
            Some(resolved) => resolved.saturating_sub(self.started_at),
            None => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                now.saturating_sub(self.started_at)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_conversation::types::{MessageType, Speaker};

    fn make_message(from: Speaker, to: Speaker, content: &str) -> ConversationMessage {
        ConversationMessage {
            from,
            to,
            content: content.to_string(),
            message_type: MessageType::Response,
            timestamp: 1234567890,
            reliability: Some(85),
            risk_level: None,
        }
    }

    #[test]
    fn test_conversation_new() {
        let conv = Conversation::new("CN-0001");
        assert_eq!(conv.id, "CN-0001");
        assert!(!conv.resolved);
        assert!(conv.messages.is_empty());
    }

    #[test]
    fn test_conversation_add_message() {
        let mut conv = Conversation::new("CN-0001");
        conv.add_message(make_message(
            Speaker::Anna,
            Speaker::Junior("Bob".to_string()),
            "Hello",
        ));

        assert_eq!(conv.messages.len(), 1);
    }

    #[test]
    fn test_conversation_resolve() {
        let mut conv = Conversation::new("CN-0001");
        conv.resolve();

        assert!(conv.resolved);
        assert!(conv.resolved_at.is_some());
    }

    #[test]
    fn test_conversation_participants() {
        let mut conv = Conversation::new("CN-0001");
        conv.add_message(make_message(
            Speaker::Anna,
            Speaker::Junior("Bob".to_string()),
            "Hello",
        ));
        conv.add_message(make_message(
            Speaker::Junior("Bob".to_string()),
            Speaker::Anna,
            "Hi there",
        ));

        let participants = conv.participants();
        assert_eq!(participants.len(), 2);
    }
}
