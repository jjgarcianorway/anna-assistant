//! Utility functions for conversation queries.

use crate::specialist_conversation::history::ConversationHistory;

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
    use crate::specialist_conversation::conversation::Conversation;

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
}
