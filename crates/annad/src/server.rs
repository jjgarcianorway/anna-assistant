//! Unix socket server for annad.
//! v0.0.159: Update check loop extracted to update_loop.rs.
//! v0.0.269: Intelligent model auto-selection with benchmarking.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anna_shared::ledger::{Ledger, LedgerEntry, LedgerEntryKind};
use anna_shared::rpc::RpcRequest;
use anna_shared::{SOCKET_PATH, STATE_DIR};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

use crate::auto_select;
use crate::hardware::probe_hardware;
use crate::health::health_check_loop;
use crate::ollama;
use crate::rpc_handler::handle_request;
use crate::snapshot_loop::snapshot_loop;
use crate::state::{create_shared_state, SharedState};
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
        // Ensure directories exist
        self.setup_directories().await?;

        // Initialize daemon
        self.initialize().await?;

        // Start update check loop
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            update_check_loop(state_clone).await;
        });

        // Start health check loop
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            health_check_loop(state_clone).await;
        });

        // v0.0.266: Start snapshot collection loop
        tokio::spawn(async move {
            snapshot_loop().await;
        });

        // Start socket server
        self.run_socket_server().await
    }

    async fn setup_directories(&self) -> Result<()> {
        // Create state directory
        fs::create_dir_all(STATE_DIR)?;

        // Create run directory for socket
        let socket_dir = Path::new(SOCKET_PATH).parent().unwrap();
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

        // Get hardware info for model selection
        let (available_ram_gb, has_gpu) = {
            let state = self.state.read().await;
            let ram_gb = state.hardware.ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            let has_gpu = state.hardware.gpu.is_some();
            (ram_gb, has_gpu)
        };

        // v0.0.269: Intelligent model auto-selection with benchmarking
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("selecting_models");
        }

        // Try auto-selection first, fall back to config defaults
        let (translator_model, specialist_model) =
            match auto_select::auto_select_models(available_ram_gb, has_gpu).await {
                Ok(result) => {
                    // Record models pulled by Anna
                    for model_name in &result.models_pulled {
                        let mut state = self.state.write().await;
                        state.ledger.add(LedgerEntry::new(
                            LedgerEntryKind::ModelPulled,
                            model_name.clone(),
                            true, // Pulled by Anna (not pre-existing)
                        ));
                    }

                    // Update state with benchmark results
                    {
                        let mut state = self.state.write().await;
                        for (model, bench) in &result.benchmarks {
                            state.add_model(
                                model,
                                "benchmarked",
                                bench.tokens_per_sec as u64,
                            );
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
                    let state = self.state.read().await;
                    (
                        state.config.llm.translator_model.clone(),
                        state.config.llm.specialist_model.clone(),
                    )
                }
            };

        // Phase: Pulling models (ensure selected models are available)
        {
            let mut state = self.state.write().await;
            state.set_llm_phase("pulling_models");
        }

        let required_models = vec![translator_model.clone(), specialist_model.clone()];
        for model_name in &required_models {
            if !ollama::has_model(model_name).await {
                info!("Pulling model: {}", model_name);
                ollama::pull_model(model_name).await?;
                let mut state = self.state.write().await;
                state.ledger.add(LedgerEntry::new(
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
            let mut state = self.state.write().await;
            state.add_model(&translator_model, "translator", 0);
            state.add_model(&specialist_model, "specialist", 0);
            if supervisor_model != translator_model && supervisor_model != specialist_model {
                state.add_model(&supervisor_model, "supervisor", 0);
            }
            // v0.0.74: Set selected model info for status display
            state.llm.translator_model = Some(translator_model.clone());
            state.llm.specialist_model = Some(specialist_model.clone());
            // Detect preferred family from model names
            let family = anna_shared::model_selector::detect_family(&specialist_model);
            state.llm.preferred_family = Some(format!("{:?}", family));
        }

        // Run benchmark on specialist model (primary inference model)
        let _throughput = ollama::benchmark(&specialist_model).await.unwrap_or(0.0);

        // Save ledger
        {
            let state = self.state.read().await;
            state.ledger.save()?;
        }

        // Mark ready
        {
            let mut state = self.state.write().await;
            state.set_llm_ready();
        }

        info!("Daemon initialized and ready");
        Ok(())
    }

    async fn run_socket_server(&self) -> Result<()> {
        let listener = UnixListener::bind(SOCKET_PATH)?;
        info!("Listening on {}", SOCKET_PATH);

        // Set socket permissions: world accessible for zero-friction UX
        // The anna group is used for directory permissions, not socket
        fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o666))?;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = self.state.clone();
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
