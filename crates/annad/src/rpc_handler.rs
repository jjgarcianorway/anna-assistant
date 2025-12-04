//! RPC request handlers with timeouts and progress tracking.

use anna_shared::ledger::LedgerEntryKind;
use anna_shared::progress::{ProgressEvent, RequestStage, TimeoutConfig};
use anna_shared::rpc::{ProbeParams, ProbeResult, RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

use crate::ollama;
use crate::probes;
use crate::service_desk;
use crate::state::SharedState;
use crate::translator;

/// Progress tracker for request handling
pub struct ProgressTracker {
    events: Vec<ProgressEvent>,
    start_time: Instant,
    current_stage: Option<RequestStage>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
            current_stage: None,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn add(&mut self, event: ProgressEvent) {
        info!("{}", event.format_debug());
        self.events.push(event);
    }

    pub fn start_stage(&mut self, stage: RequestStage, timeout_secs: u64) {
        self.current_stage = Some(stage);
        self.add(ProgressEvent::starting(stage, timeout_secs, self.elapsed_ms()));
    }

    pub fn complete_stage(&mut self, stage: RequestStage) {
        self.add(ProgressEvent::complete(stage, self.elapsed_ms()));
        self.current_stage = None;
    }

    pub fn timeout_stage(&mut self, stage: RequestStage) {
        self.add(ProgressEvent::timeout(stage, self.elapsed_ms()));
        self.current_stage = None;
    }

    pub fn events(&self) -> &[ProgressEvent] {
        &self.events
    }
}

/// Shared progress state for polling (reserved for future watchdog use)
#[allow(dead_code)]
pub type SharedProgress = Arc<RwLock<ProgressTracker>>;

#[allow(dead_code)]
pub fn create_progress_tracker() -> SharedProgress {
    Arc::new(RwLock::new(ProgressTracker::new()))
}

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
        RpcMethod::Progress => handle_progress(state, id).await,
    }
}

async fn handle_status(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let status = state.to_status();
    RpcResponse::success(id, serde_json::to_value(status).unwrap())
}

async fn handle_progress(state: SharedState, id: String) -> RpcResponse {
    let state = state.read().await;
    let events: Vec<_> = state.progress_events.iter().cloned().collect();
    RpcResponse::success(id, serde_json::to_value(events).unwrap())
}

/// Service desk pipeline with timeouts: translate -> dispatch -> specialist -> supervisor
async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let config = TimeoutConfig::default();
    let mut progress = ProgressTracker::new();

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

    // Clear previous progress events
    {
        let mut state = state.write().await;
        state.progress_events.clear();
    }

    // Step 1: Translator with timeout
    progress.start_stage(RequestStage::Translator, config.translator_secs);

    let ticket = match timeout(
        Duration::from_secs(config.translator_secs),
        translator::translate(&model, query),
    )
    .await
    {
        Ok(Ok(t)) => {
            progress.complete_stage(RequestStage::Translator);
            t
        }
        Ok(Err(e)) => {
            warn!("Translator error, using fallback: {}", e);
            progress.add(ProgressEvent::error(
                RequestStage::Translator,
                e.clone(),
                progress.elapsed_ms(),
            ));
            translator::translate_fallback(query)
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Translator);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response("translator", None, vec![]);
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    info!(
        "Translator: intent={}, domain={}, confidence={:.2}, probes={:?}",
        ticket.intent, ticket.domain, ticket.confidence, ticket.needs_probes
    );

    // Step 2: Check if clarification needed
    if let Some(question) = ticket.clarification_question.clone() {
        info!("Clarification needed: {}", question);
        save_progress(&state, &progress).await;
        let result = service_desk::create_clarification_response(ticket, &question);
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 3: Run probes with timeout
    progress.start_stage(RequestStage::Probes, config.probes_total_secs);

    let probe_results = match timeout(
        Duration::from_secs(config.probes_total_secs),
        run_probes_with_timeout(&state, &ticket, &config, &mut progress),
    )
    .await
    {
        Ok(results) => {
            progress.complete_stage(RequestStage::Probes);
            results
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response("probes", Some(ticket), vec![]);
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

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

    // Step 5: Specialist with timeout
    progress.start_stage(RequestStage::Specialist, config.specialist_secs);

    let answer = match timeout(
        Duration::from_secs(config.specialist_secs),
        ollama::chat_with_timeout(&model, &full_prompt, config.specialist_secs),
    )
    .await
    {
        Ok(Ok(response)) => {
            progress.complete_stage(RequestStage::Specialist);
            response
        }
        Ok(Err(e)) => {
            error!("Specialist LLM error: {}", e);
            progress.add(ProgressEvent::error(
                RequestStage::Specialist,
                e.to_string(),
                progress.elapsed_ms(),
            ));
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                "specialist",
                Some(ticket),
                probe_results,
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Specialist);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                "specialist",
                Some(ticket),
                probe_results,
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    // Step 6: Supervisor
    progress.start_stage(RequestStage::Supervisor, config.supervisor_secs);
    let result = service_desk::build_result(answer, ticket, probe_results);
    progress.complete_stage(RequestStage::Supervisor);

    info!(
        "Request completed: domain={}, reliability={}, probes={}",
        result.domain,
        result.reliability_score,
        result.evidence.probes_executed.len()
    );

    save_progress(&state, &progress).await;
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Run probes with individual timeouts
async fn run_probes_with_timeout(
    state: &SharedState,
    ticket: &anna_shared::rpc::TranslatorTicket,
    config: &TimeoutConfig,
    progress: &mut ProgressTracker,
) -> Vec<ProbeResult> {
    let mut results = Vec::new();

    for probe_id in &ticket.needs_probes {
        if let Some(cmd) = translator::probe_id_to_command(probe_id) {
            progress.add(ProgressEvent::probe_running(probe_id, progress.elapsed_ms()));

            // Check cache first
            let cached = {
                let state = state.read().await;
                state.get_cached_probe(cmd)
            };

            if let Some(cached_result) = cached {
                info!("Using cached probe result for: {}", cmd);
                progress.add(ProgressEvent::probe_complete(
                    probe_id,
                    cached_result.exit_code,
                    cached_result.timing_ms,
                    progress.elapsed_ms(),
                ));
                results.push(cached_result);
            } else {
                // Run probe with timeout
                let probe_start = Instant::now();
                let result = match timeout(
                    Duration::from_secs(config.probe_each_secs),
                    tokio::task::spawn_blocking({
                        let cmd = cmd.to_string();
                        move || probes::run_command_structured(&cmd)
                    }),
                )
                .await
                {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        warn!("Probe task error: {}", e);
                        ProbeResult {
                            command: cmd.to_string(),
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: format!("Task error: {}", e),
                            timing_ms: probe_start.elapsed().as_millis() as u64,
                        }
                    }
                    Err(_) => {
                        warn!("Probe timeout: {}", cmd);
                        ProbeResult {
                            command: cmd.to_string(),
                            exit_code: -1,
                            stdout: String::new(),
                            stderr: "Probe timeout".to_string(),
                            timing_ms: config.probe_each_secs * 1000,
                        }
                    }
                };

                progress.add(ProgressEvent::probe_complete(
                    probe_id,
                    result.exit_code,
                    result.timing_ms,
                    progress.elapsed_ms(),
                ));

                // Cache successful results
                if result.exit_code == 0 {
                    let mut state = state.write().await;
                    state.cache_probe(result.clone());
                }

                results.push(result);
            }
        }
    }

    // Clean expired cache
    {
        let mut state = state.write().await;
        state.clean_probe_cache();
    }

    results
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    let mut state = state.write().await;
    state.progress_events = progress.events().to_vec();
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
