//! Fast path stage - answer health/status queries without LLM (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.

use anna_shared::rpc::RpcResponse;
use anna_shared::transcript::TranscriptEvent;
use tracing::info;

use crate::fast_path_handler::{build_fast_path_result, try_fast_path_answer};
use crate::progress_tracker::ProgressTracker;

/// Try to handle query via fast path (no LLM needed).
/// Returns Some(RpcResponse) if handled, None if LLM is needed.
pub fn try_handle_fast_path(
    id: &str,
    request_id: &str,
    query: &str,
    fast_path_enabled: bool,
    snapshot_max_age_secs: u64,
    progress: &mut ProgressTracker,
) -> Option<RpcResponse> {
    if !fast_path_enabled {
        return None;
    }

    let result = try_fast_path_answer(query, snapshot_max_age_secs)?;

    info!(
        "Fast path handled: class={}, reliability={}",
        result.class, result.reliability
    );

    // Add fast path event to transcript
    let elapsed = progress.elapsed_ms();
    progress.transcript_mut().push(TranscriptEvent::fast_path(
        elapsed,
        true,
        result.class.to_string(),
        &result.trace_note,
        false, // No probes needed if we had fresh snapshot
    ));

    // Build result and return immediately
    let fast_result = build_fast_path_result(
        request_id.to_string(),
        result.answer,
        result.class,
        result.reliability,
        progress.transcript_clone(),
    );

    // Safe JSON serialization
    match serde_json::to_value(fast_result) {
        Ok(v) => Some(RpcResponse::success(id.to_string(), v)),
        Err(e) => Some(RpcResponse::error(
            id.to_string(),
            -32603,
            format!("Serialization error: {}", e),
        )),
    }
}
