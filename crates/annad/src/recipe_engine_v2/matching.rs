//! Recipe matching logic module.
//!
//! Handles matching queries to learned recipes and determining executability.

use anna_shared::recipe_engine::Recipe as LearnedRecipe;
use anna_shared::recipe_store_v2::MatchType;
use tracing::info;

use crate::recipe_engine_v2::params::{extract_missing_params, has_default_params};
use crate::recipe_engine_v2::store::get_store;

/// Minimum score to use learned recipe
const LEARNED_RECIPE_THRESHOLD: f32 = 0.7;

/// Result of checking learned recipes
#[derive(Debug)]
pub struct LearnedRecipeResult {
    pub matched: bool,
    pub recipe_id: Option<String>,
    pub recipe_name: Option<String>,
    pub score: f32,
    pub match_type: Option<MatchType>,
    pub can_execute: bool,
    pub params_needed: Vec<String>,
}

impl LearnedRecipeResult {
    pub fn no_match() -> Self {
        Self {
            matched: false,
            recipe_id: None,
            recipe_name: None,
            score: 0.0,
            match_type: None,
            can_execute: false,
            params_needed: vec![],
        }
    }
}

/// Check learned recipes for a match
pub fn check_learned_recipes(query: &str, domain: Option<&str>) -> LearnedRecipeResult {
    let store = get_store();
    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return LearnedRecipeResult::no_match(),
    };

    // Find best match
    if let Some(m) = store.best_match(query, domain, LEARNED_RECIPE_THRESHOLD) {
        if let Some(recipe) = store.get(&m.recipe_id) {
            let params_needed = extract_missing_params(query, recipe);
            let can_execute = params_needed.is_empty() || has_default_params(recipe);

            info!(
                "Learned recipe match: {} (score={:.2}, type={:?})",
                recipe.name, m.score, m.match_type
            );

            return LearnedRecipeResult {
                matched: true,
                recipe_id: Some(recipe.id.clone()),
                recipe_name: Some(recipe.name.clone()),
                score: m.score,
                match_type: Some(m.match_type),
                can_execute,
                params_needed,
            };
        }
    }

    LearnedRecipeResult::no_match()
}
