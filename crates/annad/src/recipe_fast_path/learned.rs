//! Learned recipe handling for recipe fast path.
//! v0.0.412: Integration with RecipeStoreV2 self-learning system.

use crate::recipe_engine_v2;
use anna_shared::rpc::ServiceDeskResult;

use super::types::RecipeFastPathResult;

/// v0.0.412: Execute a learned recipe and return result
pub fn execute_learned_recipe(
    result: &RecipeFastPathResult,
    query: &str,
    request_id: &str,
) -> Option<ServiceDeskResult> {
    let recipe_id = result.learned_recipe_id.as_ref()?;
    recipe_engine_v2::execute_learned_recipe(recipe_id, query, request_id)
}

/// v0.0.412: Check if this is a learned recipe match
pub fn is_learned_recipe(result: &RecipeFastPathResult) -> bool {
    result.learned_recipe_id.is_some()
}

/// Check if a recipe result can provide a direct answer (has answer_template)
pub fn can_answer_directly(result: &RecipeFastPathResult) -> bool {
    // v0.0.412: Learned recipes can always answer directly
    if result.learned_recipe_id.is_some() {
        return result.skip_llm;
    }
    result.skip_llm
        && result
            .recipe
            .as_ref()
            .map(|r| !r.answer_template.is_empty())
            .unwrap_or(false)
}
