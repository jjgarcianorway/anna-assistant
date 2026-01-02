//! Matcher extraction logic for recipe matching.

use crate::recipe_schema::{RecipeMatcher, RecipePattern};
use super::types::TicketData;
use super::id_generator::is_stop_word;

/// Extract pattern from ticket data.
pub fn extract_pattern(data: &TicketData) -> RecipePattern {
    RecipePattern {
        user_goal: data.eligibility.user_query.clone(),
        slots: data.slots.clone(),
    }
}

/// Extract matcher from ticket data.
pub fn extract_matcher(data: &TicketData) -> RecipeMatcher {
    let query = data.eligibility.user_query.to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();

    // Required keywords: nouns and key verbs from query
    let required: Vec<String> = words
        .iter()
        .filter(|w| w.len() > 3 && !is_stop_word(w))
        .take(4)
        .map(|s| s.to_string())
        .collect();

    // Optional keywords: from slots and summary
    let mut optional: Vec<String> = data.slots.values().cloned().collect();
    if let Some(summary) = &data.eligibility.specialist_summary {
        let summary_words: Vec<String> = summary
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 4 && !is_stop_word(w))
            .take(3)
            .map(String::from)
            .collect();
        optional.extend(summary_words);
    }

    // Negative keywords: detect similar but different tools
    let negative = detect_negative_keywords(&query);

    RecipeMatcher {
        required_keywords: required,
        optional_keywords: optional,
        negative_keywords: negative,
        min_confidence: 0.8,
        exact_intent: data.eligibility.intent.clone(),
    }
}

/// Detect negative keywords (things this recipe should NOT match).
fn detect_negative_keywords(query: &str) -> Vec<String> {
    let mut negatives = Vec::new();

    // Editor-specific negatives
    if query.contains("vim") && !query.contains("neovim") {
        negatives.push("neovim".into());
        negatives.push("nvim".into());
    }
    if query.contains("neovim") || query.contains("nvim") {
        negatives.push("emacs".into());
    }
    if query.contains("emacs") {
        negatives.push("vim".into());
        negatives.push("nvim".into());
    }

    // Shell-specific negatives
    if query.contains("bash") {
        negatives.push("zsh".into());
        negatives.push("fish".into());
    }
    if query.contains("zsh") {
        negatives.push("bash".into());
        negatives.push("fish".into());
    }

    negatives
}
