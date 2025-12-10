//! Unix socket server for annad.
//! v0.0.159: Update check loop extracted to update_loop.rs.
//! v0.0.269: Intelligent model auto-selection with benchmarking.
//! v0.0.281: Telemetry collector integration.
//! v0.0.310: Non-blocking model pulls - daemon ready immediately.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;

use anna_shared::ledger::{Ledger, LedgerEntry, LedgerEntryKind};
use anna_shared::rpc::RpcRequest;
use anna_shared::system_telemetry::TelemetryStore;
use anna_shared::{SOCKET_PATH, STATE_DIR};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::auto_select;
use crate::hardware::probe_hardware;
use crate::health::health_check_loop;
use crate::ollama;
use crate::rpc_handler::handle_request;
use crate::snapshot_loop::snapshot_loop;
use crate::state::{create_shared_state, SharedState};
use crate::telemetry_collector;
use crate::update_loop::update_check_loop;

pub struct Server {
    state: SharedState,
}

impl Server {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            state: create_shared_state(),
        })
    }

    pub async fn run(&self) -> Result<()> {
        // v0.0.298: Create socket EARLY so annactl can connect while initializing
        // This fixes the long wait for anna.sock after install
        self.setup_directories().await?;

        // Start socket server in background BEFORE initialization
        // The server will accept connections but return "initializing" status
        let state_for_socket = self.state.clone();
        let socket_handle = tokio::spawn(async move {
            if let Err(e) = Self::run_socket_server_impl(state_for_socket).await {
                error!("Socket server error: {}", e);
            }
        });

        // Now initialize daemon (this can be slow - model selection, pulling)
        // But clients can already connect and see "initializing" status
        self.initialize().await?;

        // v0.0.291: Enhanced background loop lifecycle logging
        // Start update check loop
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            info!("Background loop started: update_check");
            update_check_loop(state_clone).await;
            // This should never be reached unless loop panics/returns
            error!("Background loop terminated unexpectedly: update_check");
        });

        // Start health check loop
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            info!("Background loop started: health_check");
            health_check_loop(state_clone).await;
            error!("Background loop terminated unexpectedly: health_check");
        });

        // v0.0.266: Start snapshot collection loop
        tokio::spawn(async move {
            info!("Background loop started: snapshot_collector");
            snapshot_loop().await;
            error!("Background loop terminated unexpectedly: snapshot_collector");
        });

        // v0.0.281: Start telemetry collector
        let telemetry_store = Arc::new(RwLock::new(TelemetryStore::load()));
        telemetry_collector::start_collector(telemetry_store);
        info!("Telemetry collector started");

        // Wait for socket server (will run forever or until error)
        let _ = socket_handle.await;
        Ok(())
    }

    async fn setup_directories(&self) -> Result<()> {
        // Create state directory
        fs::create_dir_all(STATE_DIR)?;

        // Create run directory for socket
        // v0.0.291: Safe path handling - avoid panic on malformed paths
        let socket_dir = Path::new(SOCKET_PATH)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid socket path: {}", SOCKET_PATH))?;
        fs::create_dir_all(socket_dir)?;

        // Remove stale socket
        if Path::new(SOCKET_PATH).exists() {
            fs::remove_file(SOCKET_PATH)?;
        }

        // Record in ledger
        let mut state = self.state.write().await;
        state.ledger.add(LedgerEntry::new(
            LedgerEntryKind::DirectoryCreated,
            STATE_DIR.to_string(),
            true,
        ));

        Ok(())
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing daemon...");

        // Load existing ledger if available
        {
            let mut state = self.state.write().await;
            if let Ok(ledger) = Ledger::load() {
                state.ledger = ledger;
                info!("Loaded existing ledger");
            }
        }

        // Phase: Installing Ollama
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("installing_ollama");
        }

        if !ollama::is_installed() {
            ollama::install().await?;
            let mut state = self.state.write().await;
            state.ledger.add(LedgerEntry::new(
                LedgerEntryKind::PackageInstalled,
                "ollama".to_string(),
                true,
            ));
        }

        // Start Ollama if not running
        if !ollama::is_running().await {
            ollama::start_service().await?;
        }

        // Update Ollama status
        {
            let mut state = self.state.write().await;
            state.ollama = ollama::get_status().await;
        }

        // Phase: Probing hardware
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("probing_hardware");
        }

        let hardware = probe_hardware()?;
        {
            let mut state = self.state.write().await;
            state.hardware = hardware.clone();
        }

        // Phase: Benchmarking
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("benchmarking");
        }

        // Set benchmark result based on hardware
        {
            let mut state = self.state.write().await;
            let cpu_status = if state.hardware.cpu_cores >= 4 {
                "ok"
            } else {
                "limited"
            };
            let ram_status = if state.hardware.ram_bytes >= 8 * 1024 * 1024 * 1024 {
                "ok"
            } else {
                "limited"
            };
            let gpu_status = state
                .hardware
                .gpu
                .as_ref()
                .map(|_| "detected")
                .unwrap_or("none");
            state.set_benchmark_result(cpu_status, ram_status, gpu_status);
        }

        // v0.0.310: Mark daemon as ready EARLY with PullingModels state
        // This allows deterministic answers while models load in background
        {
            let mut state = self.state.write().await;
            state.llm.state = anna_shared::status::LlmState::PullingModels;
            state.state = anna_shared::status::DaemonState::Running;
            state.set_llm_phase("starting_model_setup");

            // Set default models from config (will be updated when auto-select completes)
            state.llm.translator_model = Some(state.config.llm.translator_model.clone());
            state.llm.specialist_model = Some(state.config.llm.specialist_model.clone());
            state.llm.junior_model = Some(state.config.llm.specialist_model.clone());
            state.llm.senior_model = Some(state.config.llm.senior_model.clone());
        }

        info!("Daemon ready for deterministic queries (models loading in background)");

        // v0.0.310: Model selection and pulling now happens in background
        let state_for_models = self.state.clone();
        tokio::spawn(async move {
            info!("Background model setup starting");
            if let Err(e) = Self::setup_models_background(state_for_models).await {
                error!("Background model setup failed: {}", e);
            }
        });

        // Save ledger
        {
            let state = self.state.read().await;
            state.ledger.save()?;
        }

        // v0.0.310: Daemon is already marked Running - initialization complete
        // Model setup continues in background
        info!("Daemon initialized (model setup in background)");
        Ok(())
    }

    /// v0.0.310: Background model selection and pulling
    /// Runs after daemon is already marked Ready, so deterministic answers work immediately
    async fn setup_models_background(state: SharedState) -> Result<()> {
        // Get hardware info for model selection
        let (available_ram_gb, has_gpu) = {
            let state_read = state.read().await;
            let ram_gb = state_read.hardware.ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            let has_gpu = state_read.hardware.gpu.is_some();
            (ram_gb, has_gpu)
        };

        // Update phase
        {
            let mut state_write = state.write().await;
            state_write.set_llm_phase("selecting_models");
        }

        // Try auto-selection first, fall back to config defaults
        let (translator_model, specialist_model) =
            match auto_select::auto_select_models(available_ram_gb, has_gpu).await {
                Ok(result) => {
                    // Record models pulled by Anna
                    for model_name in &result.models_pulled {
                        let mut state_write = state.write().await;
                        state_write.ledger.add(LedgerEntry::new(
                            LedgerEntryKind::ModelPulled,
                            model_name.clone(),
                            true,
                        ));
                    }

                    // Update state with benchmark results
                    {
                        let mut state_write = state.write().await;
                        for (model, bench) in &result.benchmarks {
                            state_write.add_model(model, "benchmarked", bench.tokens_per_sec as u64);
                        }
                    }

                    info!(
                        "Auto-selected: translator={} specialist={}",
                        result.translator.model, result.specialist.model
                    );
                    (result.translator.model, result.specialist.model)
                }
                Err(e) => {
                    warn!("Auto-selection failed: {}, using config defaults", e);
                    let state_read = state.read().await;
                    (
                        state_read.config.llm.translator_model.clone(),
                        state_read.config.llm.specialist_model.clone(),
                    )
                }
            };

        // Phase: Pulling models (ensure selected models are available)
        {
            let mut state_write = state.write().await;
            state_write.set_llm_phase("pulling_models");
        }

        let required_models = vec![translator_model.clone(), specialist_model.clone()];
        for model_name in &required_models {
            if !ollama::has_model(model_name).await {
                info!("Pulling model: {}", model_name);
                if let Err(e) = ollama::pull_model(model_name).await {
                    error!("Failed to pull model {}: {}", model_name, e);
                    continue;
                }
                let mut state_write = state.write().await;
                state_write.ledger.add(LedgerEntry::new(
                    LedgerEntryKind::ModelPulled,
                    model_name.clone(),
                    true,
                ));
            } else {
                info!("Model already available: {}", model_name);
            }
        }

        // Use translator as supervisor (fast model)
        let supervisor_model = translator_model.clone();

        // Add models to status with their roles
        {
            let mut state_write = state.write().await;
            state_write.add_model(&translator_model, "translator", 0);
            state_write.add_model(&specialist_model, "junior", 0);
            if supervisor_model != translator_model && supervisor_model != specialist_model {
                state_write.add_model(&supervisor_model, "supervisor", 0);
            }
            state_write.llm.translator_model = Some(translator_model.clone());
            state_write.llm.junior_model = Some(specialist_model.clone());
            state_write.llm.specialist_model = Some(specialist_model.clone());
            state_write.llm.senior_model = Some(state_write.config.llm.senior_model.clone());
            let family = anna_shared::model_selector::detect_family(&specialist_model);
            state_write.llm.preferred_family = Some(format!("{:?}", family));
        }

        // Run benchmark on specialist model
        let _throughput = ollama::benchmark(&specialist_model).await.unwrap_or(0.0);

        // Clean up unused models
        let installed = ollama::list_models().await.unwrap_or_default();
        let anna_pulled: Vec<String> = {
            let state_read = state.read().await;
            state_read
                .ledger
                .entries
                .iter()
                .filter(|e| matches!(e.kind, LedgerEntryKind::ModelPulled))
                .map(|e| e.target.clone())
                .collect()
        };

        for model in &installed {
            let anna_owns = anna_pulled.iter().any(|p| model.contains(p) || p.contains(model));
            if !anna_owns {
                continue;
            }
            let is_needed = required_models.iter().any(|r| model.contains(r) || r.contains(model));
            if is_needed {
                continue;
            }
            info!("Cleaning up unused model: {}", model);
            if let Ok(()) = ollama::delete_model(model).await {
                let mut state_write = state.write().await;
                state_write.ledger.add(LedgerEntry::new(
                    LedgerEntryKind::ModelDeleted,
                    model.clone(),
                    true,
                ));
            }
        }

        // Mark models as fully ready
        {
            let mut state_write = state.write().await;
            state_write.set_llm_ready();
            let _ = state_write.ledger.save();
        }

        info!("Background model setup complete - LLM fully ready");
        Ok(())
    }

    /// v0.0.298: Static method so it can run before initialization completes
    async fn run_socket_server_impl(state: SharedState) -> Result<()> {
        let listener = UnixListener::bind(SOCKET_PATH)?;
        info!("Socket available at {} (daemon still initializing)", SOCKET_PATH);

        // Set socket permissions: world accessible for zero-friction UX
        // The anna group is used for directory permissions, not socket
        fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o666))?;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(state, stream).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(state: SharedState, stream: UnixStream) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32700, "message": format!("Parse error: {}", e)},
                    "id": null
                });
                writer
                    .write_all(format!("{}\n", error_response).as_bytes())
                    .await?;
                line.clear();
                continue;
            }
        };

        let response = handle_request(state.clone(), request).await;
        let response_json = serde_json::to_string(&response)?;
        writer
            .write_all(format!("{}\n", response_json).as_bytes())
            .await?;

        line.clear();
    }

    Ok(())
}

// v0.0.159: update_check_loop moved to update_loop.rs
