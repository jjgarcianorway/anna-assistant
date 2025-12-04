//! RPC request handlers.

use anna_shared::ledger::LedgerEntryKind;
use anna_shared::rpc::{RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use tracing::{error, info};

use crate::ollama;
use crate::state::SharedState;

/// Handle an RPC request
pub async fn handle_request(state: SharedState, request: RpcRequest) -> RpcResponse {
    let id = request.id.clone();

    match request.method {
        RpcMethod::Status => handle_status(state, id).await,
        RpcMethod::Request => handle_llm_request(state, id, request.params).await,
        RpcMethod::Reset => handle_reset(state, id).await,
        RpcMethod::Uninstall => handle_uninstall(state, id).await,
        RpcMethod::Autofix => handle_autofix(state, id).await,
    }
}

async fn handle_status(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let status = state.to_status();
    RpcResponse::success(id, serde_json::to_value(status).unwrap())
}

async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    // Check if LLM is ready
    {
        let state = state.read().await;
        if state.llm.state != LlmState::Ready {
            return RpcResponse::error(
                id,
                -32002,
                format!("LLM not ready: {}", state.llm.state),
            );
        }
    }

    // Parse parameters
    let params: RequestParams = match params {
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

    // Get model name
    let model = {
        let state = state.read().await;
        state
            .llm
            .models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "llama3.2:1b".to_string())
    };

    // Build system prompt for safe, read-only operations
    let system_prompt = r#"You are Anna, a helpful local AI assistant for Linux systems.
For v0.0.1, you can only perform READ-ONLY operations. You cannot modify files or system state.
If the user asks for something that would modify the system, explain that this capability
will be added in a future version and suggest how they could do it manually.
Always be helpful, concise, and accurate. If you're unsure, say so."#;

    let full_prompt = format!("{}\n\nUser: {}", system_prompt, params.prompt);

    // Call Ollama
    match ollama::chat(&model, &full_prompt).await {
        Ok(response) => {
            info!("LLM request completed");
            RpcResponse::success(id, serde_json::json!({ "response": response }))
        }
        Err(e) => {
            error!("LLM request failed: {}", e);
            RpcResponse::error(id, -32003, format!("LLM error: {}", e))
        }
    }
}

async fn handle_reset(state: SharedState, id: String) -> RpcResponse {
    info!("Processing reset request");

    let mut state = state.write().await;

    // Reset non-base ledger entries
    state.ledger.reset_non_base();

    // Save ledger
    if let Err(e) = state.ledger.save() {
        error!("Failed to save ledger: {}", e);
        return RpcResponse::error(id, -32004, format!("Failed to save ledger: {}", e));
    }

    info!("Reset completed");
    RpcResponse::success(id, serde_json::json!({ "status": "reset_complete" }))
}

async fn handle_uninstall(state: SharedState, id: String) -> RpcResponse {
    info!("Processing uninstall request");

    let state = state.read().await;
    let ledger = &state.ledger;

    // Generate uninstall commands based on ledger
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

    // Collect models for display
    let models: Vec<String> = state.llm.models.iter().map(|m| m.name.clone()).collect();

    // Add final cleanup
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

async fn handle_autofix(state: SharedState, id: String) -> RpcResponse {
    info!("Running autofix");

    let mut fixes_applied: Vec<String> = Vec::new();

    // Check Ollama
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

    // Update state
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
