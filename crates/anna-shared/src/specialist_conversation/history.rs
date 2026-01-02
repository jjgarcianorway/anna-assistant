//! Conversation history tracking.

use crate::specialist_conversation::conversation::Conversation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conversation history tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationHistory {
    /// All conversations
    pub conversations: Vec<Conversation>,
    /// Count by department
    pub by_department: HashMap<String, u64>,
    /// Count by specialist
    pub by_specialist: HashMap<String, u64>,
    /// Total messages
    pub total_messages: u64,
}

impl ConversationHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a conversation
    pub fn add(&mut self, conv: Conversation) {
        self.total_messages += conv.messages.len() as u64;

        if let Some(dept) = &conv.department {
            *self.by_department.entry(dept.clone()).or_insert(0) += 1;
        }

        for msg in &conv.messages {
            if msg.from.is_specialist() {
                *self.by_specialist.entry(msg.from.name().to_string()).or_insert(0) += 1;
            }
        }

        self.conversations.push(conv);
    }

    /// Get recent conversations
    pub fn recent(&self, limit: usize) -> Vec<&Conversation> {
        self.conversations.iter().rev().take(limit).collect()
    }

    /// Get resolved conversations
    pub fn resolved(&self) -> Vec<&Conversation> {
        self.conversations.iter().filter(|c| c.resolved).collect()
    }

    /// Get open/active conversations
    pub fn active(&self) -> Vec<&Conversation> {
        self.conversations.iter().filter(|c| !c.resolved).collect()
    }

    /// Get conversation by ID
    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.conversations.iter().find(|c| c.id == id)
    }

    /// Total conversation count
    pub fn total_count(&self) -> usize {
        self.conversations.len()
    }

    /// Average messages per conversation
    pub fn avg_messages_per_conversation(&self) -> f64 {
        if self.conversations.is_empty() {
            return 0.0;
        }
        self.total_messages as f64 / self.conversations.len() as f64
    }

    /// Average resolution time in seconds
    pub fn avg_resolution_secs(&self) -> u64 {
        let resolved: Vec<_> = self.resolved();
        if resolved.is_empty() {
            return 0;
        }
        let total: u64 = resolved.iter().map(|c| c.duration_secs()).sum();
        total / resolved.len() as u64
    }

    /// Most active specialist
    pub fn most_active_specialist(&self) -> Option<(&str, u64)> {
        self.by_specialist
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Most active department
    pub fn most_active_department(&self) -> Option<(&str, u64)> {
        self.by_department
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_conversation::types::{ConversationMessage, MessageType, Speaker};

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
    fn test_conversation_history() {
        let mut history = ConversationHistory::new();

        let mut conv = Conversation::new("CN-0001");
        conv.department = Some("Desktop".to_string());
        conv.add_message(make_message(
            Speaker::Anna,
            Speaker::Junior("Bob".to_string()),
            "Hello",
        ));
        history.add(conv);

        assert_eq!(history.total_count(), 1);
        assert_eq!(history.total_messages, 1);
    }

    #[test]
    fn test_recent_and_active() {
        let mut history = ConversationHistory::new();

        let mut conv1 = Conversation::new("CN-0001");
        conv1.resolve();
        history.add(conv1);

        let conv2 = Conversation::new("CN-0002");
        history.add(conv2);

        assert_eq!(history.active().len(), 1);
        assert_eq!(history.resolved().len(), 1);
    }
}
