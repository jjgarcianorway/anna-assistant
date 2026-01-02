//! Triage handler - manages clarification requests (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.

use anna_shared::rpc::{RpcResponse, SpecialistDomain, TranslatorTicket};

use crate::progress_tracker::ProgressTracker;
use crate::routing_stage::RoutingResult;
use crate::state::SharedState;
use crate::theatre::TheatreContext;
use crate::triage;

use super::helpers::save_progress;
use super::request_helpers::create_clarification_response;

/// Check if immediate clarification is needed and handle it.
/// Returns Some(RpcResponse) if clarification needed, None otherwise.
pub async fn check_and_handle_clarification(
    id: &str,
    request_id: &str,
    query: &str,
    routing_result: &RoutingResult,
    ticket: &TranslatorTicket,
    classified_domain: SpecialistDomain,
    state: &SharedState,
    progress: &mut ProgressTracker,
) -> Option<RpcResponse> {
    let triage_result = routing_result.triage_result.as_ref()?;

    if !triage_result.needs_immediate_clarification {
        return None;
    }

    save_progress(state, progress).await;

    let question = triage_result
        .clarification_question
        .clone()
        .unwrap_or_else(|| triage::generate_heuristic_clarification(query));

    // v0.0.290: Create theatre and notify for clarification request
    let mut theatre = TheatreContext::new(query, classified_domain);
    theatre.ticket.pending_question = Some(question.clone());
    theatre.notify_needs_clarification();
    let _ = theatre.save();

    Some(create_clarification_response(
        id,
        request_id.to_string(),
        ticket.clone(),
        &question,
        progress.transcript_clone(),
    ))
}
