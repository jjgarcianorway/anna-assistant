//! Utility RPC handlers for status, probes, reset, uninstall, autofix, and stats.
//! v0.0.404: Reset now clears event log for consistent stats.

use anna_shared::event_log::clear_event_log;
use anna_shared::helpers::clear_helpers_store;
use anna_shared::inventory::clear_inventory;
use anna_shared::ledger::LedgerEntryKind;
use anna_shared::pending::clear_pending;
use anna_shared::recipe::clear_all_recipes;
use anna_shared::rpc::{ProbeParams, RpcResponse};
use anna_shared::snapshot::clear_snapshots;
use anna_shared::staff_stats::StaffStats;
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
/// v0.0.247: Includes live streaming events for real-time token display
pub async fn handle_progress(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let mut events = state.progress_events.clone();
    let progress_count = events.len();

    // v0.0.247: Merge in live streaming events (pushed during LLM call)
    let streaming_count = if let Ok(streaming) = state.streaming_events.lock() {
        let count = streaming.len();
        events.extend(streaming.iter().cloned());
        count
    } else {
        0
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

/// Handle stats request (v0.0.27)
/// v0.0.79: Now returns actual tracked stats from daemon state
pub async fn handle_stats(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    RpcResponse::success(id, serde_json::to_value(&state.stats).unwrap())
}

/// Handle status snapshot request (v0.0.29)
pub async fn handle_status_snapshot(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let snapshot = state.to_status_snapshot();
    RpcResponse::success(id, serde_json::to_value(snapshot).unwrap())
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
    RpcResponse::success(id, serde_json::to_value(daemon_info).unwrap())
}

// === v0.0.95: Safe Change Engine handlers ===

/// Handle PlanChange request - creates a change plan for user confirmation
pub async fn handle_plan_change(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    use anna_shared::change::plan_ensure_line;
    use anna_shared::rpc::PlanChangeParams;
    use std::path::Path;

    let params: PlanChangeParams = match params {
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

    let config_path = Path::new(&params.config_path);
    match plan_ensure_line(config_path, &params.line) {
        Ok(plan) => {
            info!("Change planned: {}", plan.summary());
            RpcResponse::success(id, serde_json::to_value(&plan).unwrap())
        }
        Err(e) => {
            error!("Failed to plan change: {}", e);
            RpcResponse::error(id, -32006, format!("Failed to plan change: {}", e))
        }
    }
}

/// Handle ApplyChange request - applies a confirmed change plan
pub async fn handle_apply_change(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    use anna_shared::change::apply_change;
    use anna_shared::rpc::ChangeParams;

    let params: ChangeParams = match params {
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

    let result = apply_change(&params.plan);
    if result.applied {
        info!("Change applied successfully");
    } else if result.was_noop {
        info!("Change was no-op (already in desired state)");
    } else if let Some(ref err) = result.error {
        error!("Change failed: {}", err);
    }
    RpcResponse::success(id, serde_json::to_value(&result).unwrap())
}

/// Handle RollbackChange request - rolls back a change using backup
pub async fn handle_rollback_change(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    use anna_shared::change::rollback;
    use anna_shared::rpc::ChangeParams;

    let params: ChangeParams = match params {
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

    let result = rollback(&params.plan);
    if result.applied {
        info!("Change rolled back successfully");
    } else if let Some(ref err) = result.error {
        error!("Rollback failed: {}", err);
    }
    RpcResponse::success(id, serde_json::to_value(&result).unwrap())
}

// === v0.0.275: LLM-generated greeting handler ===

/// Handle GenerateGreeting request - uses translator LLM to generate personalized greeting
pub async fn handle_generate_greeting(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    use crate::greeting_generator;
    use anna_shared::greeting_context::GreetingContext;

    // Parse greeting context from params
    let ctx: GreetingContext = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(ctx) => ctx,
            Err(e) => {
                warn!("Invalid greeting params: {}, using defaults", e);
                GreetingContext::default()
            }
        },
        None => GreetingContext::default(),
    };

    // Get translator model from state
    let translator_model = {
        let state = state.read().await;
        state
            .llm
            .translator_model
            .clone()
            .unwrap_or_else(|| state.config.llm.translator_model.clone())
    };

    info!(
        "Generating greeting for {} using {}",
        ctx.username, translator_model
    );

    // Generate greeting with 10 second timeout (greeting should be quick)
    let response = greeting_generator::generate_greeting(&translator_model, &ctx, 10).await;

    info!(
        "Greeting generated: {} chars, llm={}",
        response.greeting.len(),
        response.is_llm_generated
    );

    RpcResponse::success(id, serde_json::to_value(&response).unwrap())
}

// === v0.0.312: Command execution handler ===

/// Handle ExecuteCommand request - runs a user-approved shell command
/// This runs as the daemon (with elevated privileges) so commands like sudo work
pub async fn handle_execute_command(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    use anna_shared::rpc::{CommandExecutionResult, ExecuteCommandParams};
    use std::process::Command;
    use std::time::Instant;

    let params: ExecuteCommandParams = match params {
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

    info!(
        "Executing user-approved command: {} (request_id: {})",
        params.command, params.request_id
    );

    let start = Instant::now();

    // Execute the command via sh -c to support pipes, sudo, etc.
    let output = Command::new("sh").arg("-c").arg(&params.command).output();

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(output) => {
            let result = CommandExecutionResult {
                success: output.status.success(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms,
            };

            if result.success {
                info!("Command completed successfully in {}ms", duration_ms);
            } else {
                warn!(
                    "Command failed with exit code {} in {}ms",
                    result.exit_code, duration_ms
                );
            }

            RpcResponse::success(id, serde_json::to_value(&result).unwrap())
        }
        Err(e) => {
            error!("Failed to execute command: {}", e);
            let result = CommandExecutionResult {
                success: false,
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Failed to spawn command: {}", e),
                duration_ms,
            };
            RpcResponse::success(id, serde_json::to_value(&result).unwrap())
        }
    }
}
