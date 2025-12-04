//! RPC request handlers.

use anna_shared::ledger::LedgerEntryKind;
use anna_shared::rpc::{ProbeParams, RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use tracing::{error, info, warn};

use crate::ollama;
use crate::probes;
use crate::service_desk;
use crate::state::SharedState;
use crate::translator;

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

/// Service desk pipeline: translate -> dispatch -> specialist -> supervisor
async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    // Check if LLM is ready and get model name
    let model = {
        let state = state.read().await;
        if state.llm.state != LlmState::Ready {
            return RpcResponse::error(id, -32002, format!("LLM not ready: {}", state.llm.state));
        }
        state
            .llm
            .models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "llama3.2:1b".to_string())
    };

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

    let query = &params.prompt;

    // Step 1: Translator - convert query to structured ticket
    info!("Step 1: Translating query");
    let ticket = match translator::translate(&model, query).await {
        Ok(t) => t,
        Err(e) => {
            warn!("LLM translator failed, using fallback: {}", e);
            translator::translate_fallback(query)
        }
    };

    info!(
        "Translator: intent={}, domain={}, confidence={:.2}, probes={:?}",
        ticket.intent, ticket.domain, ticket.confidence, ticket.needs_probes
    );

    // Step 2: Check if clarification needed
    if let Some(question) = &ticket.clarification_question {
        info!("Clarification needed: {}", question);
        let question_owned = question.clone();
        let result = service_desk::create_clarification_response(ticket, &question_owned);
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 3: Dispatcher - run probes from ticket (with caching)
    info!("Step 3: Running probes");
    let probe_results = run_probes_with_cache(&state, &ticket).await;

    info!(
        "Probes executed: {} ({} successful)",
        probe_results.len(),
        probe_results.iter().filter(|p| p.exit_code == 0).count()
    );

    // Step 4: Build context and specialist prompt
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    let system_prompt =
        service_desk::build_specialist_prompt(ticket.domain, &context, &probe_results);
    let full_prompt = format!("{}\n\nUser: {}", system_prompt, query);

    // Step 5: Specialist - call LLM for answer
    info!("Step 5: Calling specialist LLM");
    let answer = match ollama::chat(&model, &full_prompt).await {
        Ok(response) => response,
        Err(e) => {
            error!("LLM request failed: {}", e);
            return RpcResponse::error(id, -32003, format!("LLM error: {}", e));
        }
    };

    // Step 6: Supervisor - build result with reliability scoring
    info!("Step 6: Building result with reliability scoring");
    let result = service_desk::build_result(answer, ticket, probe_results);

    info!(
        "Request completed: domain={}, reliability={}, probes={}",
        result.domain,
        result.reliability_score,
        result.evidence.probes_executed.len()
    );

    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Run probes with caching support
async fn run_probes_with_cache(
    state: &SharedState,
    ticket: &anna_shared::rpc::TranslatorTicket,
) -> Vec<anna_shared::rpc::ProbeResult> {
    let mut results = Vec::new();

    for probe_id in &ticket.needs_probes {
        if let Some(cmd) = translator::probe_id_to_command(probe_id) {
            // Check cache first
            let cached = {
                let state = state.read().await;
                state.get_cached_probe(cmd)
            };

            if let Some(cached_result) = cached {
                info!("Using cached probe result for: {}", cmd);
                results.push(cached_result);
            } else {
                // Run probe and cache result
                let result = probes::run_command_structured(cmd);

                // Cache successful results
                if result.exit_code == 0 {
                    let mut state = state.write().await;
                    state.cache_probe(result.clone());
                }

                results.push(result);
            }
        }
    }

    // Clean expired cache entries periodically
    {
        let mut state = state.write().await;
        state.clean_probe_cache();
    }

    results
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
