//! RPC request dispatcher (v0.0.830).
//!
//! v0.0.830: Fixed internal_comms - now properly converted to Transcript events.

use anna_shared::progress::{ProgressEvent, RequestStage};
use anna_shared::rpc::params::{ClaimFeedbackParams, TruthLedgerQueryParams, WebSearchParams};
use anna_shared::rpc::result::{
    EvidenceBlock, ReliabilitySignals, ServiceDeskResult, TruthLedgerClaimItem,
    TruthLedgerClaimsResult, TruthLedgerStatus, WebSearchItem, WebSearchResult,
};
use anna_shared::rpc::{RpcMethod, RpcRequest, RpcResponse, SpecialistDomain};
use anna_shared::transcript::{Actor, Transcript, TranscriptEvent};
use anna_shared::truth_ledger::Veracity;

use crate::core_loop;
use crate::feedback_handler;
use crate::handlers;
use crate::state::{SharedState, TRUTH_LEDGER_PATH};
use tracing::{info, warn};

use super::handle_web_search;
use super::llm_request::handle_llm_request;

/// Handle an RPC request
pub async fn handle_request(state: SharedState, request: RpcRequest) -> RpcResponse {
    let id = request.id.clone();

    match request.method {
        RpcMethod::Status => handlers::handle_status(state.clone(), id).await,
        // v0.0.811: Use core_loop for all requests (the simple learning path)
        RpcMethod::Request => handle_core_query(state.clone(), id, request.params).await,
        RpcMethod::Reset => handlers::handle_reset(state.clone(), id).await,
        RpcMethod::Uninstall => handlers::handle_uninstall(state.clone(), id).await,
        RpcMethod::Autofix => handlers::handle_autofix(state.clone(), id).await,
        RpcMethod::Probe => handlers::handle_probe(state.clone(), id, request.params).await,
        RpcMethod::Progress => handlers::handle_progress(state.clone(), id).await,
        RpcMethod::Stats => handlers::handle_stats(state.clone(), id).await,
        RpcMethod::StatusSnapshot => handlers::handle_status_snapshot(state.clone(), id).await,
        RpcMethod::GetDaemonInfo => handlers::handle_get_daemon_info(state.clone(), id).await,
        RpcMethod::PlanChange => handlers::handle_plan_change(id, request.params).await,
        RpcMethod::ApplyChange => handlers::handle_apply_change(id, request.params).await,
        RpcMethod::RollbackChange => handlers::handle_rollback_change(id, request.params).await,
        RpcMethod::GenerateGreeting => {
            handlers::handle_generate_greeting(state.clone(), id, request.params).await
        }
        RpcMethod::ExecuteCommand => handlers::handle_execute_command(id, request.params).await,
        RpcMethod::SubmitFeedback => {
            feedback_handler::handle_submit_feedback(state.clone(), id, request.params).await
        }
        RpcMethod::SubmitClaimFeedback => {
            handle_claim_feedback(state.clone(), id, request.params).await
        }
        RpcMethod::GetTruthLedgerStatus => handle_get_truth_ledger_status(state.clone(), id).await,
        RpcMethod::GetTruthLedgerClaims => {
            handle_get_truth_ledger_claims(state.clone(), id, request.params).await
        }
        RpcMethod::WebSearch => super::handle_web_search(id, request.params).await,
    }
}

async fn handle_claim_feedback(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let params: ClaimFeedbackParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => return RpcResponse::error(id, -32602, format!("Invalid params: {}", e)),
        },
        None => return RpcResponse::error(id, -32602, "Missing params".to_string()),
    };

    let mut state_write = state.write().await;
    let recorded = state_write
        .truth_ledger
        .add_feedback(&params.claim_text, params.positive_feedback);

    if recorded {
        info!("Recorded feedback for claim: {}", params.claim_text);
    } else {
        warn!("Claim not found in truth ledger: {}", params.claim_text);
    }

    // Save the truth ledger immediately
    if let Err(e) = state_write.truth_ledger.save(TRUTH_LEDGER_PATH) {
        warn!("Failed to save truth ledger after feedback: {}", e);
    }

    RpcResponse::success(id, serde_json::json!({"recorded": recorded}))
}

async fn handle_get_truth_ledger_status(state: SharedState, id: String) -> RpcResponse {
    let state_read = state.read().await;
    let total_claims = state_read.truth_ledger.entries.len();
    let verified_claims = state_read
        .truth_ledger
        .entries
        .iter()
        .filter(|e| e.veracity == Veracity::Verified)
        .count();
    let disputed_claims = state_read
        .truth_ledger
        .entries
        .iter()
        .filter(|e| e.veracity == Veracity::Disputed)
        .count();
    let unverified_claims = state_read
        .truth_ledger
        .entries
        .iter()
        .filter(|e| e.veracity == Veracity::Unverified)
        .count();
    let claims_with_positive_feedback = state_read
        .truth_ledger
        .entries
        .iter()
        .filter(|e| e.feedback == Some(true))
        .count();
    let claims_with_negative_feedback = state_read
        .truth_ledger
        .entries
        .iter()
        .filter(|e| e.feedback == Some(false))
        .count();

    let status = TruthLedgerStatus {
        total_claims,
        verified_claims,
        disputed_claims,
        unverified_claims,
        claims_with_positive_feedback,
        claims_with_negative_feedback,
    };

    RpcResponse::success(id, serde_json::to_value(status).unwrap_or_default())
}

async fn handle_get_truth_ledger_claims(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let query_params: TruthLedgerQueryParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => return RpcResponse::error(id, -32602, format!("Invalid params: {}", e)),
        },
        None => TruthLedgerQueryParams {
            claim_text: None,
            source: None,
            veracity: None,
            feedback: None,
        },
    };

    let state_read = state.read().await;
    let mut filtered_claims = Vec::new();

    for entry in state_read.truth_ledger.entries.iter() {
        let mut matches = true;

        if let Some(ref claim_text_filter) = query_params.claim_text {
            if !entry.claim.text.contains(claim_text_filter) {
                matches = false;
            }
        }
        if let Some(ref source_filter) = query_params.source {
            if !format!("{:?}", entry.source_metadata.source)
                .to_lowercase()
                .contains(&source_filter.to_lowercase())
            {
                matches = false;
            }
        }
        if let Some(ref veracity_filter) = query_params.veracity {
            if !format!("{:?}", entry.veracity)
                .to_lowercase()
                .contains(&veracity_filter.to_lowercase())
            {
                matches = false;
            }
        }
        if let Some(feedback_filter) = query_params.feedback {
            if entry.feedback != Some(feedback_filter) {
                matches = false;
            }
        }

        if matches {
            filtered_claims.push(TruthLedgerClaimItem {
                claim_text: entry.claim.text.clone(),
                source: format!("{:?}", entry.source_metadata.source),
                veracity: format!("{:?}", entry.veracity),
                trust_score: format!("{:?}", entry.source_metadata.trust_score),
                confidence_score: entry.confidence_score,
                feedback: entry.feedback,
                timestamp: entry.timestamp.to_rfc3339(),
            });
        }
    }

    let result = TruthLedgerClaimsResult {
        total_matching_claims: filtered_claims.len(),
        claims: filtered_claims,
    };

    RpcResponse::success(id, serde_json::to_value(result).unwrap_or_default())
}

/// v0.0.830: Handle core query using simplified core loop
/// Returns ServiceDeskResult for compatibility with existing client
/// v0.0.830: Now properly converts internal_comms to Transcript events and streams them
async fn handle_core_query(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    // Extract query from params
    let query = match params {
        Some(p) => match p.get("prompt").and_then(|v| v.as_str()) {
            Some(q) => q.to_string(),
            None => return RpcResponse::error(id, -32602, "Missing 'prompt' parameter".to_string()),
        },
        None => return RpcResponse::error(id, -32602, "Missing params".to_string()),
    };

    info!("CoreQuery: \"{}\"", query);

    // v0.0.830: Clear streaming events before new request
    {
        let state_write = state.write().await;
        let mut streaming = state_write.streaming_events.lock().await;
        streaming.clear();
    }

    // Use the new core loop
    let core_result = core_loop::handle_query(state.clone(), &query).await;

    // v0.0.830: Push internal comms as progress events for real-time display
    {
        let streaming_events = state.read().await.streaming_events.clone();
        let mut streaming = streaming_events.lock().await;
        for comm in &core_result.internal_comms {
            let event = ProgressEvent::internal_comms(
                RequestStage::Specialist,
                comm.from.clone(),
                comm.message.clone(),  // Clone to convert &String -> String -> DiagnosticText
                comm.elapsed_ms,
            );
            streaming.push(event);
        }
    }

    // Determine domain from source
    let domain = match &core_result.source {
        core_loop::AnswerSource::Specialist { name, .. } => {
            match name.as_str() {
                "system" => SpecialistDomain::System,
                "network" => SpecialistDomain::Network,
                "storage" => SpecialistDomain::Storage,
                "services" => SpecialistDomain::Services,
                "packages" => SpecialistDomain::Packages,
                "desktop" => SpecialistDomain::Desktop,
                "security" => SpecialistDomain::Security,
                _ => SpecialistDomain::System,
            }
        }
        _ => SpecialistDomain::System,
    };

    // Determine staff assignment
    let (staff_id, assigned_staff) = match &core_result.source {
        core_loop::AnswerSource::Recipe => (Some("anna".to_string()), Some("Anna (from recipe)".to_string())),
        core_loop::AnswerSource::Specialist { name, learned } => {
            let suffix = if *learned { " (learned)" } else { "" };
            (Some(format!("{}_specialist", name)), Some(format!("{} Specialist{}", capitalize(name), suffix)))
        }
        core_loop::AnswerSource::Failed => (None, None),
    };

    // v0.0.830: Build transcript from internal comms (properly this time!)
    let mut transcript = Transcript::new();
    for comm in &core_result.internal_comms {
        // Map "from" field to Actor enum variant
        let from_actor = match comm.from.to_lowercase().as_str() {
            "anna" => Actor::Anna,
            "junior" => Actor::Junior,
            "senior" => Actor::Senior,
            "translator" => Actor::Translator,
            "dispatcher" => Actor::Dispatcher,
            "supervisor" => Actor::Supervisor,
            _ if comm.from.contains("Specialist") => Actor::Specialist,
            _ => Actor::Specialist, // Default to Specialist for named staff
        };
        let event = TranscriptEvent::message(
            comm.elapsed_ms,
            from_actor,
            Actor::You,  // Messages are directed to the user
            &comm.message,
        );
        transcript.push(event);
    }
    // Add the final answer
    transcript.push(TranscriptEvent::final_answer(
        core_result.elapsed_ms,
        &core_result.answer,
    ));

    // Build ServiceDeskResult for compatibility
    let result = ServiceDeskResult {
        request_id: uuid::Uuid::new_v4().to_string(),
        case_number: core_result.recipe_id.clone().map(|_| format!("CN-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))),
        assigned_staff,
        staff_id,
        answer: core_result.answer,
        validated: core_result.reliability >= 80,
        reliability_score: core_result.reliability,
        reliability_signals: ReliabilitySignals {
            translator_confident: true,
            probe_coverage: true,
            answer_grounded: core_result.reliability >= 60,
            no_invention: core_result.reliability >= 70,
            clarification_not_needed: true,
        },
        reliability_explanation: None,
        domain,
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript,
        execution_trace: None,
        proposed_change: None,
        proposed_changes: vec![],
        feedback_request: None,
    };

    match serde_json::to_value(result) {
        Ok(v) => RpcResponse::success(id, v),
        Err(e) => RpcResponse::error(id, -32603, format!("Serialization error: {}", e)),
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
