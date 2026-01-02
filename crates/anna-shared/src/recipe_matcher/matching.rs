//! Main recipe matching logic.

use crate::recipe_index::{tokenize, RecipeIndex};
use std::collections::BTreeSet;

use super::substitutions::extract_substitutions;
use super::threshold::dynamic_threshold;
use super::types::{MatchResult, MIN_MATCHING_TOKENS};

/// Match query against learned recipes
///
/// Returns the best matching recipe if score > threshold, else None.
/// The translator should call this BEFORE escalating to the specialist.
/// v0.0.373: Uses dynamic thresholds based on recipe maturity/reliability.
pub fn match_recipe(query: &str, index: &RecipeIndex) -> Option<MatchResult> {
    let query_tokens: BTreeSet<String> = tokenize(query).into_iter().collect();

    if query_tokens.len() < MIN_MATCHING_TOKENS {
        return None;
    }

    // Search recipes
    let matches = index.search_recipes(query, 5);

    if matches.is_empty() {
        return None;
    }

    // Get best match
    let (recipe, raw_score) = matches.into_iter().next()?;

    // Normalize score to 0-100
    let max_possible = (query_tokens.len() * 3) as u32 + 10; // rough estimate
    let score = ((raw_score as f32 / max_possible as f32) * 100.0).min(100.0) as u32;

    // v0.0.373: Use dynamic threshold based on recipe maturity
    let threshold = dynamic_threshold(&recipe);

    // Check if strong enough (use half of dynamic threshold for early filtering)
    if score < threshold / 2 {
        return None;
    }

    // Compute matched tokens
    let recipe_tokens: BTreeSet<String> = recipe
        .intent_tags
        .iter()
        .chain(recipe.targets.iter())
        .flat_map(|s| tokenize(s))
        .chain(tokenize(&recipe.signature.query_pattern))
        .collect();

    let matched_tokens: Vec<String> = query_tokens
        .iter()
        .filter(|t| recipe_tokens.contains(*t))
        .cloned()
        .collect();

    if matched_tokens.len() < MIN_MATCHING_TOKENS {
        return None;
    }

    // v0.0.373: Determine high confidence using dynamic threshold
    let high_confidence = score >= threshold && matched_tokens.len() >= 3 && recipe.is_mature();

    // Extract substitutions (e.g., different package name, different editor)
    let substitutions = extract_substitutions(query, &recipe);

    Some(MatchResult {
        recipe,
        score,
        matched_tokens,
        high_confidence,
        substitutions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_index() {
        let index = RecipeIndex::new();
        let result = match_recipe("install htop", &index);
        assert!(result.is_none());
    }
}
