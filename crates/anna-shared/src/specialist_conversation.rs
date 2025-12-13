//! Specialist Conversation Display - Phase 78
//!
//! Tracks and displays conversations between Anna and specialists.
//! VISION.md shows the "fly on the wall" experience of internal communications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Format a conversation for display (fly-on-the-wall style)
pub fn format_conversation(conv: &Conversation) -> String {
    let mut lines = vec![format!("--- Internal Communication ---")];
    lines.push(format!("Case: {}", conv.id));
    if let Some(dept) = &conv.department {
        lines.push(format!("Department: {}", dept));
    }
    lines.push(String::new());

    for msg in &conv.messages {
        let reliability = msg
            .reliability
            .map(|r| format!(" [{}% reliable]", r))
            .unwrap_or_default();

        lines.push(format!(
            "{} to {}: {}{}",
            msg.from.name(),
            msg.to.name(),
            msg.content,
            reliability
        ));
        lines.push(String::new());
    }

    if conv.resolved {
        lines.push("--- Conversation Resolved ---".to_string());
    }

    lines.join("\n")
}

/// Format conversation history for display
pub fn format_conversation_history(history: &ConversationHistory) -> String {
    let mut lines = vec!["=== Conversation History ===".to_string()];
    lines.push(String::new());

    if history.conversations.is_empty() {
        lines.push("No conversations recorded yet.".to_string());
        return lines.join("\n");
    }

    // Summary stats
    lines.push(format!("Total conversations: {}", history.total_count()));
    lines.push(format!("Total messages: {}", history.total_messages));
    lines.push(format!(
        "Avg messages/conversation: {:.1}",
        history.avg_messages_per_conversation()
    ));
    lines.push(format!(
        "Avg resolution time: {}s",
        history.avg_resolution_secs()
    ));

    // Active/resolved
    let active = history.active().len();
    let resolved = history.resolved().len();
    lines.push(format!("Active: {} | Resolved: {}", active, resolved));

    // Most active
    if let Some((specialist, count)) = history.most_active_specialist() {
        lines.push(String::new());
        lines.push(format!("Most active specialist: {} ({} messages)", specialist, count));
    }

    if let Some((dept, count)) = history.most_active_department() {
        lines.push(format!("Most active department: {} ({} cases)", dept, count));
    }

    // Recent conversations
    let recent = history.recent(3);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent conversations:".to_string());
        for conv in recent {
            let status = if conv.resolved { "[resolved]" } else { "[active]" };
            let dept = conv.department.as_deref().unwrap_or("General");
            lines.push(format!(
                "  {} - {} msgs - {} {}",
                conv.id,
                conv.messages.len(),
                dept,
                status
            ));
        }
    }

    lines.join("\n")
}

/// Format conversation history compact
pub fn format_conversation_history_compact(history: &ConversationHistory) -> String {
    format!(
        "Conversations: {} ({} msgs) | Active: {} | Resolved: {}",
        history.total_count(),
        history.total_messages,
        history.active().len(),
        history.resolved().len()
    )
}

/// Format conversation history one-line
pub fn format_conversation_history_oneline(history: &ConversationHistory) -> String {
    format!(
        "{} conversations ({} resolved)",
        history.total_count(),
        history.resolved().len()
    )
}

/// Check if query is about conversations
pub fn is_conversation_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "conversation",
        "internal comm",
        "specialist chat",
        "anna dialog",
        "case history",
        "ticket convers",
        "messages between",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about conversations
pub fn conversation_fun_fact(history: &ConversationHistory) -> String {
    if history.conversations.is_empty() {
        return "No conversations yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has had {} conversations with specialists.",
            history.total_count()
        ),
        format!(
            "{} messages have been exchanged in total.",
            history.total_messages
        ),
        {
            if let Some((specialist, _)) = history.most_active_specialist() {
                format!("{} is the most active specialist.", specialist)
            } else {
                "No specialist activity recorded yet.".to_string()
            }
        },
        format!(
            "Average of {:.1} messages per conversation.",
            history.avg_messages_per_conversation()
        ),
    ];

    facts[history.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_format_conversation() {
        let mut conv = Conversation::new("CN-0001");
        conv.department = Some("Desktop".to_string());
        conv.add_message(make_message(
            Speaker::Anna,
            Speaker::Junior("Bob".to_string()),
            "Can you help?",
        ));

        let output = format_conversation(&conv);
        assert!(output.contains("Internal Communication"));
        assert!(output.contains("CN-0001"));
        assert!(output.contains("Anna to Bob"));
    }

    #[test]
    fn test_is_conversation_query() {
        assert!(is_conversation_query("show conversation history"));
        assert!(is_conversation_query("internal communications"));
        assert!(is_conversation_query("specialist chat logs"));
        assert!(!is_conversation_query("what is my disk space?"));
    }

    #[test]
    fn test_conversation_fun_fact() {
        let mut history = ConversationHistory::new();
        history.add(Conversation::new("CN-0001"));

        let fact = conversation_fun_fact(&history);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut history = ConversationHistory::new();
        let mut conv = Conversation::new("CN-0001");
        conv.resolve();
        history.add(conv);

        let compact = format_conversation_history_compact(&history);
        assert!(compact.contains("Conversations: 1"));

        let oneline = format_conversation_history_oneline(&history);
        assert!(oneline.contains("1 conversations"));
    }
}
