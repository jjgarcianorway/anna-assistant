//! RPC request handlers.

use anna_shared::ledger::LedgerEntryKind;
use anna_shared::rpc::{
    Capabilities, HardwareSummary, ProbeParams, ProbeType, RequestParams,
    RpcMethod, RpcRequest, RpcResponse, RuntimeContext,
};
use anna_shared::status::LlmState;
use anna_shared::VERSION;
use std::collections::HashMap;
use tracing::{error, info};

use crate::ollama;
use crate::probes;
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
        RpcMethod::Probe => handle_probe(state, id, request.params).await,
    }
}

async fn handle_status(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let status = state.to_status();
    RpcResponse::success(id, serde_json::to_value(status).unwrap())
}

/// Build runtime context from current state
fn build_runtime_context(state: &crate::state::DaemonStateInner) -> RuntimeContext {
    let hw = &state.hardware;
    RuntimeContext {
        version: VERSION.to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: hw.cpu_model.clone(),
            cpu_cores: hw.cpu_cores,
            ram_gb: hw.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            gpu: hw.gpu.as_ref().map(|g| g.model.clone()),
            gpu_vram_gb: hw.gpu.as_ref().map(|g| g.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
        },
        probes: HashMap::new(),
    }
}

/// Build grounded system prompt with runtime context
fn build_system_prompt(context: &RuntimeContext, probe_results: &HashMap<String, String>) -> String {
    let mut prompt = format!(
        r#"You are Anna, a local AI assistant running on this Linux machine.

=== RUNTIME CONTEXT (AUTHORITATIVE - DO NOT CONTRADICT) ===
Version: {}
Daemon: running
Capabilities:
  - Read system info: {}
  - Run probes: {}
  - Modify files: {}
  - Install packages: {}

Hardware (from system probe):
  - CPU: {} ({} cores)
  - RAM: {:.1} GB"#,
        context.version,
        context.capabilities.can_read_system_info,
        context.capabilities.can_run_probes,
        context.capabilities.can_modify_files,
        context.capabilities.can_install_packages,
        context.hardware.cpu_model,
        context.hardware.cpu_cores,
        context.hardware.ram_gb,
    );

    if let Some(gpu) = &context.hardware.gpu {
        if let Some(vram) = context.hardware.gpu_vram_gb {
            prompt.push_str(&format!("\n  - GPU: {} ({:.1} GB VRAM)", gpu, vram));
        } else {
            prompt.push_str(&format!("\n  - GPU: {}", gpu));
        }
    } else {
        prompt.push_str("\n  - GPU: none");
    }

    // Add probe results if any
    if !probe_results.is_empty() {
        prompt.push_str("\n\n=== PROBE RESULTS ===");
        for (name, result) in probe_results {
            prompt.push_str(&format!("\n[{}]\n{}", name, result));
        }
    }

    prompt.push_str(r#"

=== GROUNDING RULES (MANDATORY) ===
1. NEVER invent or guess information not in the runtime context above.
2. For hardware questions (CPU, RAM, GPU): Answer DIRECTLY from the hardware section above.
3. For process/memory/disk questions: Use the probe results above if available.
4. If data is missing: Say exactly what data is missing and offer to probe for it.
5. NEVER suggest manual commands like `lscpu`, `free`, `ps` etc. when you have the data above.
6. NEVER claim capabilities you don't have (check the Capabilities section).
7. ALWAYS use the exact version shown above when discussing Anna's version.

=== END CONTEXT ===

Answer the user's question using ONLY the data provided above. Be helpful and concise."#);

    prompt
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

    // Build runtime context
    let (context, model) = {
        let state = state.read().await;
        let ctx = build_runtime_context(&state);
        let model = state
            .llm
            .models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "llama3.2:1b".to_string());
        (ctx, model)
    };

    // Determine if we need to run probes based on the query
    let mut probe_results: HashMap<String, String> = HashMap::new();
    let prompt_lower = params.prompt.to_lowercase();

    // Auto-run relevant probes based on query keywords
    if prompt_lower.contains("memory") || prompt_lower.contains("ram") || prompt_lower.contains("process") {
        if let Ok(result) = probes::run_probe(&ProbeType::TopMemory) {
            probe_results.insert("top_memory_processes".to_string(), result);
        }
    }
    if prompt_lower.contains("cpu") && prompt_lower.contains("process") {
        if let Ok(result) = probes::run_probe(&ProbeType::TopCpu) {
            probe_results.insert("top_cpu_processes".to_string(), result);
        }
    }
    if prompt_lower.contains("disk") || prompt_lower.contains("storage") || prompt_lower.contains("space") {
        if let Ok(result) = probes::run_probe(&ProbeType::DiskUsage) {
            probe_results.insert("disk_usage".to_string(), result);
        }
    }
    if prompt_lower.contains("network") || prompt_lower.contains("interface") || prompt_lower.contains("ip") {
        if let Ok(result) = probes::run_probe(&ProbeType::NetworkInterfaces) {
            probe_results.insert("network_interfaces".to_string(), result);
        }
    }

    // Build grounded system prompt
    let system_prompt = build_system_prompt(&context, &probe_results);
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

async fn handle_probe(
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

async fn handle_reset(state: SharedState, id: String) -> RpcResponse {
    info!("Processing reset request");

    let mut state = state.write().await;
    state.ledger.reset_non_base();

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

async fn handle_autofix(state: SharedState, id: String) -> RpcResponse {
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
