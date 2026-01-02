//! Config recipe matching logic.

use crate::recipe_index::RecipeIndex;

use super::threshold::dynamic_threshold;
use super::types::MatchResult;

/// Try to find a recipe for a config action
///
/// Looks for recipes that can be adapted for similar config changes.
/// E.g., "enable syntax highlighting in nano" can use vim syntax recipe
/// with path/command substitutions.
pub fn match_config_recipe(
    intent: &str, // e.g., "enable syntax highlighting"
    target: &str, // e.g., "nano"
    index: &RecipeIndex,
) -> Option<MatchResult> {
    // Build search query from intent + target
    let query = format!("{} {}", intent, target);

    // First try exact match with target
    let matches = index.search_recipes(&query, 5);

    for (recipe, score) in &matches {
        // Check if recipe applies to this target
        if recipe
            .targets
            .iter()
            .any(|t| t.to_lowercase() == target.to_lowercase())
        {
            return Some(MatchResult {
                recipe: recipe.clone(),
                score: *score,
                matched_tokens: vec![intent.to_string(), target.to_string()],
                high_confidence: true,
                substitutions: vec![],
            });
        }
    }

    // Then try similar recipes that could be adapted
    // v0.0.373: Use dynamic threshold
    for (recipe, score) in matches {
        let threshold = dynamic_threshold(&recipe);
        if recipe.is_config_edit() && score >= threshold / 2 {
            // Can adapt this recipe for different target
            let substitutions = vec![("target".to_string(), target.to_string())];

            return Some(MatchResult {
                recipe,
                score,
                matched_tokens: vec![intent.to_string()],
                high_confidence: false, // Needs verification
                substitutions,
            });
        }
    }

    None
}
