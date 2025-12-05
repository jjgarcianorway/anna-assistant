//! Utility RPC handlers for status, probes, reset, uninstall, autofix, and stats.

use anna_shared::helpers::clear_helpers_store;
use anna_shared::ledger::LedgerEntryKind;
use anna_shared::pending::clear_pending;
use anna_shared::recipe::clear_all_recipes;
use anna_shared::rpc::{ProbeParams, RpcResponse};
use anna_shared::snapshot::clear_snapshots;
use anna_shared::stats::GlobalStats;
use tracing::{error, info, warn};

use crate::ollama;
use crate::probes;
use crate::state::SharedState;

/// Handle status request
pub async fn handle_status(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let status = state.to_status();
    RpcResponse::success(id, serde_json::to_value(status).unwrap())
}

/// Handle progress request
pub async fn handle_progress(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let events = state.progress_events.to_vec();
    RpcResponse::success(id, serde_json::to_value(events).unwrap())
}

/// Handle probe request
pub async fn handle_probe(
    _state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let params: ProbeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    match probes::run_probe(&params.probe_type) {
        Ok(result) => {
            info!("Probe {:?} completed", params.probe_type);
            RpcResponse::success(id, serde_json::json!({ "result": result }))
        }
        Err(e) => {
            error!("Probe failed: {}", e);
            RpcResponse::error(id, -32005, format!("Probe error: {}", e))
        }
    }
}

/// Handle reset request (v0.0.28: true state wipe)
pub async fn handle_reset(state: SharedState, id: String) -> RpcResponse {
    info!("Processing reset request - true state wipe");

    // 1. Reset ledger (existing behavior)
    let mut state = state.write().await;
    state.ledger.reset_non_base();

    if let Err(e) = state.ledger.save() {
        error!("Failed to save ledger: {}", e);
        return RpcResponse::error(id, -32004, format!("Failed to save ledger: {}", e));
    }
    info!("Ledger reset complete");

    // 2. Clear recipes store (v0.0.28)
    if let Err(e) = clear_all_recipes() {
        warn!("Failed to clear recipes: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Recipes cleared");
    }

    // 3. Clear helpers store (v0.0.28)
    if let Err(e) = clear_helpers_store() {
        warn!("Failed to clear helpers store: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Helpers store cleared");
    }

    // 4. Clear snapshots store (v0.0.36)
    if let Err(e) = clear_snapshots() {
        warn!("Failed to clear snapshots: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Snapshots cleared");
    }

    // 5. Clear pending clarification (v0.0.36)
    if let Err(e) = clear_pending() {
        warn!("Failed to clear pending clarification: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Pending clarification cleared");
    }

    info!("Reset completed - all learned data cleared");
    RpcResponse::success(id, serde_json::json!({
        "status": "reset_complete",
        "cleared": ["ledger", "recipes", "helpers", "snapshots", "pending"]
    }))
}

/// Handle uninstall request
pub async fn handle_uninstall(state: SharedState, id: String) -> RpcResponse {
    info!("Processing uninstall request");

    let state = state.read().await;
    let ledger = &state.ledger;

    let mut commands: Vec<String> = Vec::new();

    for entry in ledger.entries.iter().rev() {
        match entry.kind {
            LedgerEntryKind::ModelPulled => {
                commands.push(format!("ollama rm {}", entry.target));
            }
            LedgerEntryKind::FileCreated => {
                commands.push(format!("rm -f {}", entry.target));
            }
            LedgerEntryKind::DirectoryCreated => {
                commands.push(format!("rmdir --ignore-fail-on-non-empty {}", entry.target));
            }
            LedgerEntryKind::ServiceEnabled => {
                commands.push(format!("systemctl disable {}", entry.target));
                commands.push(format!("systemctl stop {}", entry.target));
            }
            _ => {}
        }
    }

    let models: Vec<String> = state.llm.models.iter().map(|m| m.name.clone()).collect();

    commands.push("systemctl stop annad".to_string());
    commands.push("systemctl disable annad".to_string());
    commands.push("rm -f /usr/local/bin/annactl".to_string());
    commands.push("rm -f /usr/local/bin/annad".to_string());
    commands.push("rm -f /etc/systemd/system/annad.service".to_string());
    commands.push("rm -rf /etc/anna".to_string());
    commands.push("rm -rf /var/lib/anna".to_string());
    commands.push("rm -rf /var/log/anna".to_string());
    commands.push("systemctl daemon-reload".to_string());

    RpcResponse::success(
        id,
        serde_json::json!({
            "status": "uninstall_prepared",
            "commands": commands,
            "helpers": {
                "ollama": state.ollama.installed,
                "models": models
            }
        }),
    )
}

/// Handle autofix request
pub async fn handle_autofix(state: SharedState, id: String) -> RpcResponse {
    info!("Running autofix");

    let mut fixes_applied: Vec<String> = Vec::new();

    if !ollama::is_installed() {
        info!("Autofix: Ollama not installed, installing...");
        if let Err(e) = ollama::install().await {
            return RpcResponse::error(id, -32002, format!("Failed to install Ollama: {}", e));
        }
        fixes_applied.push("Installed Ollama".to_string());
    }

    if !ollama::is_running().await {
        info!("Autofix: Ollama not running, starting...");
        if let Err(e) = ollama::start_service().await {
            return RpcResponse::error(id, -32002, format!("Failed to start Ollama: {}", e));
        }
        fixes_applied.push("Started Ollama service".to_string());
    }

    {
        let mut state = state.write().await;
        state.ollama = ollama::get_status().await;
    }

    info!("Autofix completed: {} fixes", fixes_applied.len());
    RpcResponse::success(
        id,
        serde_json::json!({
            "status": "autofix_complete",
            "fixes_applied": fixes_applied
        }),
    )
}

/// Handle stats request (v0.0.27)
pub async fn handle_stats(_state: SharedState, id: String) -> RpcResponse {
    // TODO: Track stats in daemon state, for now return empty stats
    let stats = GlobalStats::new();
    RpcResponse::success(id, serde_json::to_value(stats).unwrap())
}

/// Handle status snapshot request (v0.0.29)
pub async fn handle_status_snapshot(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let snapshot = state.to_status_snapshot();
    RpcResponse::success(id, serde_json::to_value(snapshot).unwrap())
}
