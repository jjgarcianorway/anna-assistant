//! Dialogue utilities - Phase 89
//!
//! Utility functions for dialogue handling.

use super::types::Dialogue;

/// Check if query is about dialogue
pub fn is_dialogue_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "show dialogue",
        "show conversation",
        "internal communication",
        "what did they say",
        "fly on the wall",
        "show discussion",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate dialogue fun fact
pub fn dialogue_fun_fact(dialogue: &Dialogue) -> String {
    if dialogue.turns.is_empty() {
        return "No dialogue recorded yet.".to_string();
    }

    let facts = [
        format!("This dialogue has {} turns.", dialogue.turn_count()),
        format!(
            "{} internal exchanges occurred.",
            dialogue.internal_turns().len()
        ),
        format!(
            "{} messages to/from the user.",
            dialogue.external_turns().len()
        ),
    ];

    facts[dialogue.turn_count() % facts.len()].clone()
}
