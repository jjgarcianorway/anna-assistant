//! Recipe execution module.
//!
//! Handles executing learned recipes and recording results.

use anna_shared::recipe_executor::{ExecutionContext, RecipeExecutor};
use anna_shared::rpc::ServiceDeskResult;

use crate::recipe_engine_v2::params::extract_param_value;
use crate::recipe_engine_v2::result_builder::build_learned_recipe_result;
use crate::recipe_engine_v2::store::get_store;

/// Execute a learned recipe
pub fn execute_learned_recipe(
    recipe_id: &str,
    query: &str,
    request_id: &str,
) -> Option<ServiceDeskResult> {
    let store = get_store();
    let mut store = match store.write() {
        Ok(s) => s,
        Err(_) => return None,
    };

    let recipe = store.get(recipe_id)?.clone();

    // Build execution context with extracted params
    let mut ctx = ExecutionContext::default();
    ctx.ticket_id = Some(request_id.to_string());
    ctx.recipe_id = Some(recipe_id.to_string());

    // Extract parameters from query
    for param in &recipe.parameters {
        if let Some(value) =
            extract_param_value(&query.to_lowercase(), &param.name, &param.extraction_hint)
        {
            ctx.params.insert(param.name.clone(), value);
        } else if let Some(default) = &param.default {
            ctx.params.insert(param.name.clone(), default.clone());
        }
    }

    // Execute
    let executor = RecipeExecutor::new();
    let result = executor.execute(&recipe, &mut ctx);

    // Update recipe stats
    if let Some(r) = store.get_mut(recipe_id) {
        if result.success {
            r.record_success();
        } else {
            r.record_failure();
        }
    }
    let _ = store.save();

    // Build result
    Some(build_learned_recipe_result(
        request_id.to_string(),
        &recipe,
        result,
        query,
    ))
}
