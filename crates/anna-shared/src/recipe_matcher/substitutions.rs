//! Parameter substitution logic for adapting recipes to new queries.

use crate::recipe::Recipe;
use crate::recipe_index::tokenize;

use super::helpers::{
    looks_like_editor_name, looks_like_package_name, looks_like_service_name, looks_like_target,
};

/// Extract parameter substitutions between query and recipe
pub fn extract_substitutions(query: &str, recipe: &Recipe) -> Vec<(String, String)> {
    let mut subs = Vec::new();

    let query_tokens = tokenize(query);
    let pattern_tokens = tokenize(&recipe.signature.query_pattern);

    // Find tokens in query that aren't in pattern (likely parameters)
    for qt in &query_tokens {
        if !pattern_tokens.contains(qt) {
            // Try to identify what kind of parameter this is
            if looks_like_package_name(qt) {
                subs.push(("package".to_string(), qt.clone()));
            } else if looks_like_service_name(qt) {
                subs.push(("service".to_string(), qt.clone()));
            } else if looks_like_editor_name(qt) {
                subs.push(("editor".to_string(), qt.clone()));
            }
        }
    }

    subs
}

/// Extract substitutions for action recipes
pub fn extract_action_substitutions(
    action: &str,
    target: &str,
    recipe: &Recipe,
) -> Vec<(String, String)> {
    let mut subs = Vec::new();

    // Extract original target from recipe
    let pattern_tokens = tokenize(&recipe.signature.query_pattern);

    // The new target is a substitution for whatever was in the original
    for pt in pattern_tokens {
        if pt != action && looks_like_target(&pt) {
            subs.push((pt, target.to_string()));
            break;
        }
    }

    subs
}
