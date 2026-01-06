//! Status-related RPC handlers.
//! Handles status, progress, stats, daemon_info, and status_snapshot requests.

use super::types::*;

/// Handle status request
pub async fn handle_status(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let status = state.to_status();
    RpcResponse::success(id, serde_json::to_value(status).unwrap_or_default())
}

/// Handle progress request
/// v0.0.247: Includes live streaming events for real-time token display
/// v0.0.825: Use tokio::sync::Mutex for async-safe streaming events
pub async fn handle_progress(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let mut events = state.progress_events.clone();
    let progress_count = events.len();

    // v0.0.247: Merge in live streaming events (pushed during LLM call)
    // v0.0.825: Use async lock for tokio::sync::Mutex
    let streaming_events = state.streaming_events.clone();
    drop(state); // Release state lock before acquiring streaming lock

    let streaming_count = {
        let streaming = streaming_events.lock().await;
        let count = streaming.len();
        events.extend(streaming.iter().cloned());
        count
    };

    // Sort by timestamp to maintain temporal order
    events.sort_by_key(|e| e.elapsed_ms);

    // v0.0.248: Debug logging for progress polling verification
    if streaming_count > 0 || progress_count > 0 {
        tracing::debug!(
            "Progress poll: {} progress + {} streaming = {} total events",
            progress_count,
            streaming_count,
            events.len()
        );
    }

    RpcResponse::success(id, serde_json::to_value(events).unwrap_or_default())
}

/// Handle stats request (v0.0.27)
/// v0.0.79: Now returns actual tracked stats from daemon state
pub async fn handle_stats(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    RpcResponse::success(id, serde_json::to_value(&state.stats).unwrap_or_default())
}

/// Handle status snapshot request (v0.0.29)
pub async fn handle_status_snapshot(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let snapshot = state.to_status_snapshot();
    RpcResponse::success(id, serde_json::to_value(snapshot).unwrap_or_default())
}

/// v0.0.73: Handle GetDaemonInfo request - returns daemon version info
pub async fn handle_get_daemon_info(state: SharedState, id: String) -> RpcResponse {
    use anna_shared::rpc::DaemonInfo;
    use anna_shared::version::VersionInfo;

    let state = state.read().await;
    let daemon_info = DaemonInfo {
        version_info: VersionInfo::current(),
        pid: state.pid,
        uptime_secs: state.started_at.elapsed().as_secs(),
    };
    RpcResponse::success(id, serde_json::to_value(daemon_info).unwrap_or_default())
}
