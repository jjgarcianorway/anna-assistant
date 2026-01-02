//! Routing handler - manages recipe and translator routing (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.

use anna_shared::rpc::{RpcResponse, TranslatorTicket};

use crate::comms::CommsGenerator;
use crate::progress_tracker::ProgressTracker;
use crate::recipe_fast_path;
use crate::result_stage::wrap_with_theatre;
use crate::router::DeterministicRoute;
use crate::routing_stage::{enforce_probe_spine, route_query, RoutingResult};
use crate::state::SharedState;

use super::request_helpers::RequestConfig;

/// Handle routing stage (recipe check + LLM translator).
/// Returns Some(RpcResponse) if recipe can answer directly, None otherwise.
pub async fn handle_routing_stage(
    state: &SharedState,
    id: &str,
    request_id: &str,
    query: &str,
    det_route: &DeterministicRoute,
    config: &RequestConfig,
    progress: &mut ProgressTracker,
) -> (Option<RpcResponse>, TranslatorTicket, RoutingResult) {
    // Step 2: v0.0.167 - Route query through recipe check or LLM translator
    let routing_result = route_query(
        state,
        query,
        det_route,
        &config.llm_config,
        &config.translator_model,
        config.hw_cores,
        config.hw_ram_gb,
        config.has_gpu,
        config.debug_mode,
        progress,
    )
    .await;

    // Handle recipe direct answer
    if let Some(ref recipe_result) = routing_result.recipe_result {
        if recipe_fast_path::can_answer_directly(recipe_result) {
            let recipe = recipe_result.recipe.as_ref().unwrap();
            progress.add_translator_message(&format!(
                "Recipe match: {} (score {})",
                recipe.id, recipe_result.score
            ));

            let result = recipe_fast_path::build_recipe_result(
                request_id.to_string(),
                recipe,
                &recipe_result.matched_tokens,
                progress.transcript_clone(),
                query,
            );

            let response = wrap_with_theatre(id.to_string(), result, None);
            let ticket = routing_result.ticket.clone();
            return (Some(response), ticket, routing_result);
        }
    }

    let mut ticket = routing_result.ticket.clone();

    // Step 2.5: v0.0.167 - Enforce probe spine constraints
    enforce_probe_spine(&mut ticket, query, det_route);

    (None, ticket, routing_result)
}

/// Handle team communications (dispatch and acknowledgement).
pub async fn handle_team_comms(
    det_route_class: &str,
    classified_domain: &str,
    request_id: &str,
    query: &str,
    progress: &mut ProgressTracker,
    state: &SharedState,
) -> CommsGenerator {
    use crate::comms::team_from_query_class;

    // v0.0.148: Create comms generator for fly-on-wall experience
    // v0.0.266: Use query class for team routing
    let team = team_from_query_class(det_route_class, classified_domain);
    let mut comms = CommsGenerator::new(team, request_id).with_query(query);

    // v0.0.254: Anna dispatches to team and junior acknowledges
    comms.dispatch_async(progress).await;
    comms.junior_ack_async(progress).await;
    super::helpers::save_progress(state, progress).await;

    comms
}
