//! Timeout response handling for request pipeline.
//! v0.0.150: Extracted from rpc_handler.rs for modularization.

use anna_shared::rpc::{RpcResponse, ServiceDeskResult, SpecialistDomain};
use anna_shared::transcript::Transcript;
use tracing::info;

use crate::fast_path_handler::{build_fast_path_result, force_fast_path_fallback, is_health_query};

/// Build a response for a timed-out request.
/// v0.0.40: For health queries, uses fast path fallback instead of generic timeout.
/// v0.0.141: Friendlier timeout message with helpful suggestions.
pub fn make_timeout_response(
    id: String,
    request_id: String,
    timeout_secs: u64,
    query: Option<&str>,
) -> RpcResponse {
    // v0.0.40: For health queries, use fast path fallback instead of timeout message
    if let Some(q) = query {
        if is_health_query(q) {
            if let Some(fallback) = force_fast_path_fallback(q) {
                info!("Using fast path fallback for health query on timeout");
                let result = build_fast_path_result(
                    request_id.clone(),
                    fallback.answer,
                    fallback.class,
                    fallback.reliability,
                    Transcript::default(),
                );
                // v0.0.291: Safe JSON serialization - handle unlikely serialization failures
                return match serde_json::to_value(result) {
                    Ok(v) => RpcResponse::success(id, v),
                    Err(e) => RpcResponse::error(id, -32603, format!("Serialization error: {}", e)),
                };
            }
        }
    }

    // v0.0.141: Friendlier timeout message with helpful suggestions
    let answer = build_timeout_message(timeout_secs);

    let result = ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        validated: false,      // v0.0.298: Timeout responses are not validated
        reliability_score: 20, // Low but not zero - we provided info
        reliability_signals: anna_shared::rpc::ReliabilitySignals::default(),
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence: anna_shared::rpc::EvidenceBlock::default(),
        needs_clarification: false, // Never ask to rephrase
        clarification_question: None,
        clarification_request: None,
        transcript: Transcript::default(),
        execution_trace: Some(anna_shared::trace::ExecutionTrace::global_timeout(
            timeout_secs,
        )),
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };
    // v0.0.291: Safe JSON serialization - handle unlikely serialization failures
    match serde_json::to_value(result) {
        Ok(v) => RpcResponse::success(id, v),
        Err(e) => RpcResponse::error(id, -32603, format!("Serialization error: {}", e)),
    }
}

/// Build a user-friendly timeout message with suggestions.
fn build_timeout_message(timeout_secs: u64) -> String {
    format!(
        "I'm taking longer than expected ({}s). Let me help you differently:\n\n\
         **Try these quick queries instead:**\n\
         - \"what cpu\" - CPU information\n\
         - \"disk space\" - Storage usage\n\
         - \"memory\" - RAM usage\n\
         - \"running services\" - Active services\n\
         - \"network interfaces\" - Network info\n\n\
         These bypass the LLM and give instant answers.\n\n\
         *Tip: Run `annactl status` to check LLM health.*",
        timeout_secs
    )
}
