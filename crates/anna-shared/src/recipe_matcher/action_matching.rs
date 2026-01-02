//! Action recipe matching logic.

use crate::recipe::{RecipeAction, RecipeKind};
use crate::recipe_index::RecipeIndex;

use super::substitutions::extract_action_substitutions;
use super::threshold::dynamic_threshold;
use super::types::MatchResult;

/// Try to find a recipe for a package/service action
///
/// Looks for recipes that can be adapted for similar package operations.
/// E.g., "install htop" can use "install vim" recipe with package substitution.
pub fn match_action_recipe(
    action: &str, // e.g., "install", "restart"
    target: &str, // e.g., "htop", "docker"
    index: &RecipeIndex,
) -> Option<MatchResult> {
    // Search for similar action recipes
    let query = format!("{} {}", action, target);
    let matches = index.search_recipes(&query, 10);

    for (recipe, score) in matches {
        // Check if this is an action recipe
        match &recipe.kind {
            RecipeKind::Query => {
                // Check if query pattern contains the same action verb
                if recipe.signature.query_pattern.contains(action) {
                    let substitutions = extract_action_substitutions(action, target, &recipe);
                    let threshold = dynamic_threshold(&recipe);

                    return Some(MatchResult {
                        recipe,
                        score,
                        matched_tokens: vec![action.to_string()],
                        high_confidence: score >= threshold,
                        substitutions,
                    });
                }
            }
            _ => {
                // Config edit recipes can also match action patterns
                if let RecipeAction::EnsureLine { line } = &recipe.action {
                    if line.contains(action) || line.contains(target) {
                        return Some(MatchResult {
                            recipe,
                            score,
                            matched_tokens: vec![action.to_string(), target.to_string()],
                            high_confidence: true,
                            substitutions: vec![],
                        });
                    }
                }
            }
        }
    }

    None
}
