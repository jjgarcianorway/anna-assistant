//! State management handlers for reset, uninstall, and autofix operations.

use super::types::*;
use anna_shared::event_log::clear_event_log;
use anna_shared::helpers::clear_helpers_store;
use anna_shared::inventory::clear_inventory;
use anna_shared::ledger::LedgerEntryKind;
use anna_shared::pending::clear_pending;
use anna_shared::recipe::clear_all_recipes;
use anna_shared::snapshot::clear_snapshots;
use anna_shared::staff_stats::StaffStats;

use crate::ollama;

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

    // 6. Clear inventory cache (v0.0.39)
    if let Err(e) = clear_inventory() {
        warn!("Failed to clear inventory cache: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Inventory cache cleared");
    }

    // 7. Clear staff stats (v0.0.306)
    if let Err(e) = StaffStats::clear() {
        warn!("Failed to clear staff stats: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Staff stats cleared");
    }

    // 8. Clear event log (v0.0.404: fixes stats inconsistency after reset)
    if let Err(e) = clear_event_log() {
        warn!("Failed to clear event log: {}", e);
        // Not fatal, continue with reset
    } else {
        info!("Event log cleared");
    }

    info!("Reset completed - all learned data cleared");
    RpcResponse::success(
        id,
        serde_json::json!({
            "status": "reset_complete",
            "cleared": ["ledger", "recipes", "helpers", "snapshots", "pending", "inventory", "staff_stats", "event_log"]
        }),
    )
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
