//! RPC request dispatcher (v0.0.200).

use anna_shared::rpc::{RpcMethod, RpcRequest, RpcResponse};

use crate::handlers;
use crate::state::SharedState;

use super::llm_request::handle_llm_request;

/// Handle an RPC request
pub async fn handle_request(state: SharedState, request: RpcRequest) -> RpcResponse {
    let id = request.id.clone();

    match request.method {
        RpcMethod::Status => handlers::handle_status(state, id).await,
        RpcMethod::Request => handle_llm_request(state, id, request.params).await,
        RpcMethod::Reset => handlers::handle_reset(state, id).await,
        RpcMethod::Uninstall => handlers::handle_uninstall(state, id).await,
        RpcMethod::Autofix => handlers::handle_autofix(state, id).await,
        RpcMethod::Probe => handlers::handle_probe(state, id, request.params).await,
        RpcMethod::Progress => handlers::handle_progress(state, id).await,
        RpcMethod::Stats => handlers::handle_stats(state, id).await,
        RpcMethod::StatusSnapshot => handlers::handle_status_snapshot(state, id).await,
        RpcMethod::GetDaemonInfo => handlers::handle_get_daemon_info(state, id).await,
        RpcMethod::PlanChange => handlers::handle_plan_change(id, request.params).await,
        RpcMethod::ApplyChange => handlers::handle_apply_change(id, request.params).await,
        RpcMethod::RollbackChange => handlers::handle_rollback_change(id, request.params).await,
        RpcMethod::GenerateGreeting => {
            handlers::handle_generate_greeting(state, id, request.params).await
        }
    }
}
