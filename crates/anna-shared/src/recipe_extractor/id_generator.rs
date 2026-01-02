//! Recipe ID generation logic.

use crate::recipe_eligibility::RecipeType;
use super::types::TicketData;

/// Generate a recipe ID from ticket data.
pub fn generate_recipe_id(data: &TicketData, recipe_type: Option<RecipeType>) -> String {
    let intent = data.eligibility.intent.as_deref().unwrap_or("unknown");
    let domain = data.eligibility.domain.as_deref().unwrap_or("general");

    // Extract key nouns from query
    let query = data.eligibility.user_query.to_lowercase();
    let key_words: Vec<&str> = query
        .split_whitespace()
        .filter(|w| w.len() > 3 && !is_stop_word(w))
        .take(3)
        .collect();

    let type_suffix = match recipe_type {
        Some(RecipeType::ConfigChange) => "config",
        Some(RecipeType::RepeatableDiagnostic) => "check",
        Some(RecipeType::SimpleFix) => "fix",
        Some(RecipeType::ServiceAction) => "service",
        Some(RecipeType::PackageAction) => "package",
        None => "action",
    };

    if key_words.is_empty() {
        format!(
            "{}_{}_{}_{}",
            domain,
            intent,
            type_suffix,
            &data.ticket_id[..8.min(data.ticket_id.len())]
        )
    } else {
        format!("{}_{}_{}", domain, key_words.join("_"), type_suffix)
    }
}

pub fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
        "can", "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above", "below", "between",
        "under", "again", "further", "then", "once", "here", "there", "when", "where", "why",
        "how", "all", "each", "few", "more", "most", "other", "some", "such", "only", "own",
        "same", "than", "too", "very", "just", "also", "now",
    ];
    STOP_WORDS.contains(&word)
}
