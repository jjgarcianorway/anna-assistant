//! Formatting functions for conversation display.

use crate::specialist_conversation::conversation::Conversation;
use crate::specialist_conversation::history::ConversationHistory;

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
