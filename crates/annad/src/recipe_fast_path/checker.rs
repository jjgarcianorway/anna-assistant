//! Recipe fast path checker - main logic for matching queries to recipes.
//! v0.0.412: Check learned recipes FIRST (RecipeStoreV2) before hardcoded.

use crate::recipe_builtins::{
    check_cron_recipes, check_docker_recipes, check_git_recipes, check_shell_recipes,
    check_ssh_recipes, check_systemd_recipes,
};
use crate::recipe_engine_v2;
use anna_shared::recipe_index::RecipeIndex;
use anna_shared::recipe_matcher::match_recipe;
use tracing::info;

use super::types::RecipeFastPathResult;

/// Check recipe index for a matching recipe
pub fn check_recipe_fast_path(query: &str, index: &RecipeIndex) -> RecipeFastPathResult {
    // v0.0.412: Check LEARNED recipes FIRST (self-learning system)
    let learned = recipe_engine_v2::check_learned_recipes(query, None);
    if learned.matched && learned.can_execute {
        info!(
            "Learned recipe match: {} (score={:.2}, type={:?})",
            learned.recipe_name.as_deref().unwrap_or("?"),
            learned.score,
            learned.match_type
        );
        // Create a minimal RecipeFastPathResult for learned recipes
        return RecipeFastPathResult {
            matched: true,
            ticket: None, // Will be executed via execute_learned_recipe
            recipe: None, // Learned recipes use Recipe from RecipeStoreV2
            score: (learned.score * 100.0) as u32,
            matched_tokens: vec![learned.recipe_name.clone().unwrap_or_default()],
            skip_llm: true,
            learned_recipe_id: learned.recipe_id,
        };
    }

    // Then try the general recipe matcher
    if let Some(result) = match_recipe(query, index) {
        info!(
            "Recipe match found: score={}, tokens={:?}, skip_llm={}",
            result.score,
            result.matched_tokens,
            result.score >= super::types::RECIPE_SKIP_LLM_THRESHOLD && result.high_confidence
        );
        return RecipeFastPathResult::from_recipe(result);
    }

    // Third, check built-in shell recipes
    if let Some(result) = check_shell_recipes(query) {
        info!(
            "Shell recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Third, check built-in git recipes
    if let Some(result) = check_git_recipes(query) {
        info!("Git recipe match found");
        return result;
    }

    // Fourth, check built-in SSH recipes (v0.0.104)
    if let Some(result) = check_ssh_recipes(query) {
        info!(
            "SSH recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Fifth, check built-in systemd recipes (v0.0.233)
    if let Some(result) = check_systemd_recipes(query) {
        info!(
            "Systemd recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Sixth, check built-in cron recipes (v0.0.234)
    if let Some(result) = check_cron_recipes(query) {
        info!(
            "Cron recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    // Seventh, check built-in Docker recipes (v0.0.235)
    if let Some(result) = check_docker_recipes(query) {
        info!(
            "Docker recipe match found: {}",
            result
                .recipe
                .as_ref()
                .map(|r| r.id.clone())
                .unwrap_or_default()
        );
        return result;
    }

    RecipeFastPathResult::no_match()
}
